import Hls from 'hls.js';
import type { Chapter } from './types.js';
import {i18n} from "@/stores/i18n.svelte";
import {appConfig} from "@/stores/config.svelte";
import type { SubtitleSettings } from './subtitles/SubtitleSettings.svelte.js';
import {invoke} from "@tauri-apps/api/core";

export interface PlayerCallbacks {
    onTimeUpdate?: (data: { currentTime: number; duration: number; paused: boolean }) => void;
    onPlay?: () => void;
    onPause?: () => void;
    onSeek?: (time: number) => void;
    onEnded?: () => void;
}

export class PlayerController {
    paused          = $state(true);
    currentTime     = $state(0);
    duration        = $state(0);
    buffered        = $state(0);        // 0–1 fraction
    isBuffering     = $state(false);
    controlsVisible = $state(true);
    showSkipButton  = $state(false);
    skipTargetTime  = $state(0);
    skipLabel       = $state('');

    hlsError = $state<{ message: string; retrying: boolean } | null>(null);
    isReady = $state(false);

    #video:    HTMLVideoElement | null = null;
    #hls:      Hls | null = null;
    #hideTimer: ReturnType<typeof setTimeout> | null = null;
    #hasSeeked = false;

    #callbacks: PlayerCallbacks = {};
    #chapters:  Chapter[] = [];
    #initialTime = 0;

    qualityLevels   = $state<{ id: number; label: string }[]>([]);
    currentQuality  = $state<number>(-1);   // -1 = Auto

    audioTracks     = $state<{ id: number; label: string }[]>([]);
    currentAudio    = $state<number>(0);

    subtitleTracks = $state<{ id: string; srclang: string; label: string; url: string }[]>([]);
    currentSubtitle = $state<string>('-1');

    volume = $state(1);
    muted  = $state(false);

    subAutoApplied   = false;
    audioAutoApplied = false;

    #pendingSeekTime: number | null = null;
    #lastKnownTime   = 0;

    #mediaRecoveryAttempted = false;
    #currentSrc = '';
    #restartTimer: ReturnType<typeof setTimeout> | null = null;
    #restartAttempts = 0;
    static readonly #MAX_RESTART_ATTEMPTS = 3;

    #rootEl:        HTMLElement    | null = null;
    #subtitleEl:    HTMLDivElement | null = null;
    #subtitleSettings: SubtitleSettings  | null = null;
    #activeCueTrack:   TextTrack         | null = null;
    #cueChangeHandler: (() => void)      | null = null;

    attachRoot(el: HTMLElement) {
        this.#rootEl = el;
    }

    attachSubtitleOverlay(el: HTMLDivElement) {
        this.#subtitleEl = el;
    }

    attachSubtitleSettings(s: SubtitleSettings) {
        this.#subtitleSettings = s;
    }

    attachVideo(video: HTMLVideoElement) {
        this.#video = video;
        video.volume = this.volume;
        video.muted  = this.muted;
    }

    loadSrc(src: string) {
        this.#resetPlaybackState();
        if (!this.#video) return;
        this.#createHls(src);
    }

    setCallbacks(cb: PlayerCallbacks) {
        this.#callbacks = cb;
    }

    setChapters(chapters: Chapter[]) {
        this.#chapters = chapters;
    }

    setInitialTime(t: number) {
        this.#initialTime = t;
    }

    destroy() {
        this.#hls?.destroy();
        this.#hls = null;
        if (this.#hideTimer) clearTimeout(this.#hideTimer);
        if (this.#restartTimer) clearTimeout(this.#restartTimer);
        if (this.#activeCueTrack && this.#cueChangeHandler) {
            this.#activeCueTrack.removeEventListener('cuechange', this.#cueChangeHandler);
        }
        this.#activeCueTrack   = null;
        this.#cueChangeHandler = null;
    }

    #createHls(src: string) {
        this.#currentSrc = src;

        if (this.#video && !this.#video.paused) {
            this.#video.pause();
        }

        this.#hls?.destroy();
        this.#hls = null;

        const video = this.#video!;

        if (Hls.isSupported()) {
            this.#hls = new Hls({
                maxBufferLength: 60,
                maxMaxBufferLength: 120,
                maxBufferSize: 60 * 1000 * 1000,
                maxBufferHole: 0.5,
                lowLatencyMode: false,
                enableWorker: true,
                startLevel: -1,
                abrEwmaFastLive: 3,
                abrEwmaSlowLive: 9,
                abrEwmaFastVoD: 3,
                abrEwmaSlowVoD: 9,
            });

            this.#hls.loadSource(src);
            this.#hls.attachMedia(video);

            this.#hls.on(Hls.Events.MANIFEST_PARSED, (_e, data) => {
                if (data.levels.length > 1) {
                    this.qualityLevels = [
                        { id: -1, label: 'Auto' },
                        ...data.levels.map((l, i) => ({
                            id: i,
                            label: l.height ? `${l.height}p` : `Level ${i}`
                        }))
                    ];
                } else {
                    this.qualityLevels = [];
                }
                this.currentQuality = -1;
            });

            this.#hls.on(Hls.Events.ERROR, (_e, data) => {
                console.error('[HLS Error]', {
                    type: data.type,
                    details: data.details,
                    fatal: data.fatal,
                    url: data.url,
                    response: data.response,
                    error: data.error?.message,
                });

                if (!data.fatal) return;

                if (data.type === Hls.ErrorTypes.MEDIA_ERROR) {
                    if (data.details === Hls.ErrorDetails.BUFFER_INCOMPATIBLE_CODECS_ERROR && this.#mediaRecoveryAttempted) {
                        console.error('[HLS] Codec unsupported by this device, aborting playback');
                        this.#abortUnsupported();
                        return;
                    }

                    if (!this.#mediaRecoveryAttempted) {
                        // First attempt: let hls.js try to recover
                        this.#mediaRecoveryAttempted = true;
                        if (data.details === Hls.ErrorDetails.BUFFER_INCOMPATIBLE_CODECS_ERROR) {
                            console.warn('[HLS] Codec mismatch, swapping audio codec...');
                            this.#hls!.swapAudioCodec();
                        }
                        console.warn('[HLS] Media error recovery attempt...');
                        this.#hls!.recoverMediaError();
                        // onPlaying will reset #mediaRecoveryAttempted if recovery succeeds
                    } else {
                        // Recovery already tried and failed — full restart
                        this.#scheduleRestart();
                    }

                } else if (data.type === Hls.ErrorTypes.NETWORK_ERROR) {
                    switch (data.details) {
                        case Hls.ErrorDetails.MANIFEST_LOAD_ERROR:
                        case Hls.ErrorDetails.MANIFEST_LOAD_TIMEOUT:
                            console.error('[HLS] Manifest load failed, cannot recover');
                            this.isBuffering = false;
                            this.hlsError = { message: 'Stream error', retrying: false };
                            break;
                        default:
                            // Fragment or other network errors — full restart
                            this.#scheduleRestart();
                    }

                } else {
                    this.#scheduleRestart();
                }
            });

            this.#hls.on(Hls.Events.AUDIO_TRACKS_UPDATED, (_e, data) => {
                this.audioTracks = data.audioTracks.map((t, i) => ({
                    id: i,
                    label: t.name || t.lang || `Track ${i + 1}`
                }));
                this.currentAudio = this.#hls!.audioTrack ?? 0;
            });

            this.#hls.on(Hls.Events.AUDIO_TRACK_SWITCHED, (_e, data) => {
                this.currentAudio = data.id;
            });

        } else if (video.canPlayType('application/vnd.apple.mpegurl')) {
            video.src = src;
        }
    }

    #abortUnsupported() {
        if (this.#restartTimer) {
            clearTimeout(this.#restartTimer);
            this.#restartTimer = null;
        }
        this.#hls?.destroy();
        this.#hls = null;
        this.isBuffering = false;
        this.hlsError = { message: 'This stream is not supported on this device', retrying: false };
    }

    #scheduleRestart() {
        if (this.#restartTimer) return; // already scheduled

        this.#restartAttempts++;
        if (this.#restartAttempts > PlayerController.#MAX_RESTART_ATTEMPTS) {
            console.error('[HLS] Max restart attempts reached, aborting');
            this.#abortUnsupported();
            return;
        }

        console.error(`[HLS] Scheduling stream restart in 3s (attempt ${this.#restartAttempts}/${PlayerController.#MAX_RESTART_ATTEMPTS})`);
        this.#hls?.destroy();
        this.#hls = null;
        this.hlsError = { message: 'Stream error', retrying: true };

        // Save position so we can resume after restart
        const resumeTime = this.#lastKnownTime;

        this.#restartTimer = setTimeout(() => {
            this.#restartTimer = null;
            if (!this.#video) return;
            this.hlsError = null;
            this.#mediaRecoveryAttempted = false;
            // Queue a seek to resume position after canplay
            if (resumeTime > 0) this.#pendingSeekTime = resumeTime;
            this.#createHls(this.#currentSrc);
        }, 3000);
    }

    onCanPlay() {
        const v = this.#video;
        if (!v) return;

        // Apply initial resume time only once per source load
        if (!this.#hasSeeked) {
            if (this.#initialTime > 0) v.currentTime = this.#initialTime;
            this.#hasSeeked = true;
        }

        // Flush any seek that was queued while video was in a bad state
        if (this.#pendingSeekTime !== null) {
            v.currentTime = this.#pendingSeekTime;
            this.#callbacks.onSeek?.(this.#pendingSeekTime);
            this.#pendingSeekTime = null;
        }

        this.isBuffering = false;
        this.isReady = true;
        this.#applySubtitleTrack();
    }

    onTimeUpdate() {
        const v = this.#video;
        if (!v) return;

        this.currentTime = v.currentTime;
        this.duration    = v.duration || 0;
        this.paused      = v.paused;

        this.#callbacks.onTimeUpdate?.({
            currentTime: this.currentTime,
            duration:    this.duration,
            paused:      v.paused,
        });

        if (v.currentTime > 0 && isFinite(v.duration) && v.duration > 0) {
            this.#lastKnownTime = v.currentTime;
        }

        this.#tickChapters(v.currentTime);
    }

    onProgress() {
        const v = this.#video;
        if (!v || v.buffered.length === 0) return;
        this.buffered = v.buffered.end(v.buffered.length - 1) / (v.duration || 1);
    }

    onEnded() {
        this.paused          = true;
        this.controlsVisible = true;
        if (this.#hideTimer) clearTimeout(this.#hideTimer);
        this.#callbacks.onEnded?.();
    }

    onWaiting() { this.isBuffering = true; }

    onPlaying() {
        this.isBuffering = false;
        this.#mediaRecoveryAttempted = false;
        this.#restartAttempts = 0;
        this.paused = false;
    }

    nudgeControls() {
        this.controlsVisible = true;
        this.#scheduleHide();
    }

    #scheduleHide() {
        if (this.#hideTimer) clearTimeout(this.#hideTimer);
        if (!this.paused) {
            this.#hideTimer = setTimeout(() => {
                this.controlsVisible = false;
            }, 500);
        }
    }

    togglePlay() {
        const v = this.#video;
        if (!v || !this.#hls && !v.src && !v.currentSrc) return;

        if (v.paused) {
            v.play().then(() => {
                this.paused = false;
                this.#callbacks.onPlay?.();
                this.#scheduleHide();
            }).catch((err: DOMException) => {
                if (err.name !== 'AbortError') console.error('play() failed:', err);
            });
        } else {
            v.pause();
            this.paused          = true;
            this.controlsVisible = true;
            if (this.#hideTimer) clearTimeout(this.#hideTimer);
            this.#callbacks.onPause?.();
        }
    }

    seek(time: number) {
        const v = this.#video;
        if (!v) return;

        const durationOk = isFinite(v.duration) && v.duration > 0;
        if (!durationOk) {
            this.#pendingSeekTime = time;
            this.currentTime = time; // update UI optimistically
            return;
        }

        this.#pendingSeekTime = null;
        v.currentTime    = time;
        this.currentTime = time;
        this.#callbacks.onSeek?.(time);
    }

    seekBy(seconds: number) {
        const v = this.#video;
        if (!v) return;
        // Fall back to tracked values when video element is in a bad state
        const duration = (isFinite(v.duration) && v.duration > 0) ? v.duration : this.duration;
        const current  = (isFinite(v.duration) && v.duration > 0) ? v.currentTime : this.#lastKnownTime;
        const t = Math.max(0, Math.min(duration, current + seconds));
        this.seek(t);
    }

    skipChapter() {
        this.seek(this.skipTargetTime);
        this.showSkipButton = false;
    }

    toggleFullscreen() {
        const rootEl = this.#rootEl;
        const isFs = !!document.fullscreenElement;

        if (isFs) {
            document.exitFullscreen().catch(() => {});
            invoke('exit_fullscreen').catch(() => {});
        } else if (rootEl?.requestFullscreen) {
            rootEl.requestFullscreen({ navigationUI: 'hide' })
                .then(() => {
                    invoke('enter_fullscreen').catch(() => {});
                })
                .catch(() => {});
        } else {
            const video = rootEl?.querySelector('video') as any;
            video?.webkitEnterFullscreen?.();
        }
    }

    #tickChapters(t: number) {
        const ch = this.#chapters.find(c => t >= c.start && t < c.end);
        if (!ch) { this.showSkipButton = false; return; }

        const norm    = ch.title.toLowerCase().replace(/[^a-z0-9]/g, '');
        const isIntro = norm.includes('opening') || norm.includes('op') || norm.includes('intro');
        const isOutro = norm.includes('ending')  || norm.includes('ed') || norm.includes('outro');

        if (isIntro || isOutro) {
            const cfg = appConfig.data?.player;
            if ((isIntro && cfg?.autoSkipIntro) || (isOutro && cfg?.autoSkipOutro)) {
                this.seek(ch.end);
                this.showSkipButton = false;
                return;
            }
            this.showSkipButton = true;
            this.skipTargetTime = ch.end;
            this.skipLabel      = isIntro ? i18n.t("watch.skip_op") : i18n.t("watch.skip_ed");
        } else {
            this.showSkipButton = false;
        }
    }

    #resetPlaybackState() {
        this.isReady         = false;
        this.#hasSeeked      = false;
        this.currentTime     = 0;
        this.duration        = 0;
        this.buffered        = 0;
        this.paused          = true;
        this.showSkipButton  = false;
        this.controlsVisible = true;
        this.qualityLevels   = [];
        this.audioTracks     = [];
        this.subtitleTracks  = [];
        this.currentQuality  = -1;
        this.currentAudio    = 0;
        this.currentSubtitle = '-1';
        this.hlsError        = null;
        this.#pendingSeekTime        = null;
        this.#lastKnownTime          = 0;
        this.#mediaRecoveryAttempted = false;
        this.#restartAttempts = 0;
        if (this.#restartTimer) {
            clearTimeout(this.#restartTimer);
            this.#restartTimer = null;
        }
        if (this.#hideTimer) clearTimeout(this.#hideTimer);
    }

    setQuality(id: number) {
        if (!this.#hls) return;
        this.#hls.currentLevel = id;
        this.currentQuality = id;
    }

    setAudioTrack(id: number) {
        if (!this.#hls) return;
        this.#hls.audioTrack = id;
        this.currentAudio = id;
    }

    setSubtitles(subtitles: import('./types.js').Subtitle[]) {
        this.subtitleTracks = subtitles.map((s, i) => ({
            id: String(i),
            srclang: s.id,
            label: s.language || s.id,
            url: s.url,
        }));
        this.currentSubtitle = '-1';
    }

    setSubtitleTrack(id: string) {
        this.currentSubtitle = id;
        this.#applySubtitleTrack();
    }

    #applySubtitleTrack() {
        const v = this.#video;
        if (!v) return;

        if (this.#activeCueTrack && this.#cueChangeHandler) {
            this.#activeCueTrack.removeEventListener('cuechange', this.#cueChangeHandler);
        }
        this.#activeCueTrack   = null;
        this.#cueChangeHandler = null;

        if (this.#subtitleEl) this.#subtitleEl.innerHTML = '';

        for (let i = 0; i < v.textTracks.length; i++) {
            v.textTracks[i].mode = 'hidden';
        }

        const activeIdx = this.subtitleTracks.findIndex(s => s.id === this.currentSubtitle);
        if (activeIdx === -1 || !this.#subtitleEl) return;

        const track = v.textTracks[activeIdx];
        if (!track) return;

        track.mode = 'hidden';
        this.#activeCueTrack = track;

        this.#cueChangeHandler = () => {
            const el = this.#subtitleEl;
            if (!el) return;

            const cue = track.activeCues?.[0] as VTTCue | undefined;

            if (!cue) {
                el.innerHTML = '';
                return;
            }

            const span = document.createElement('span');
            if (this.#subtitleSettings) {
                span.setAttribute('style', this.#subtitleSettings.containerStyle);
            }

            const frag = cue.getCueAsHTML?.();
            if (frag) {
                span.appendChild(frag);
            } else {
                span.innerHTML = cue.text.replace(/\n/g, '<br>');
            }

            el.innerHTML = '';
            el.appendChild(span);
        };

        track.addEventListener('cuechange', this.#cueChangeHandler);
    }

    setVolume(v: number) {
        const vol = Math.max(0, Math.min(1, v));
        this.volume = vol;
        if (this.#video) this.#video.volume = vol;
        if (vol > 0) this.muted = false;
    }

    toggleMute() {
        this.muted = !this.muted;
        if (this.#video) this.#video.muted = this.muted;
    }

    toggleControls() {
        if (this.controlsVisible) {
            this.controlsVisible = false;
            if (this.#hideTimer) clearTimeout(this.#hideTimer);
        } else {
            this.controlsVisible = true;
            if (this.#hideTimer) clearTimeout(this.#hideTimer);
            this.#hideTimer = setTimeout(() => {
                this.controlsVisible = false;
            }, 3000);
        }
    }
}