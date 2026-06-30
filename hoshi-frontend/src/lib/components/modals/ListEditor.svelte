<script lang="ts">
    import * as Dialog from "@/components/ui/dialog";
    import { Input } from "@/components/ui/input";
    import { Label } from "@/components/ui/label";
    import { Button } from "@/components/ui/button";
    import * as Popover from "@/components/ui/popover";
    import { Calendar } from "@/components/ui/calendar";
    import { Textarea } from "@/components/ui/textarea";
    import { Checkbox } from "@/components/ui/checkbox";
    import { listApi } from "@/api/list/list";
    import type { EnrichedListEntry, EntrySource, ListEntryChange, ListStatus, UpsertEntryBody } from "@/api/list/types";
    import { toast } from "svelte-sonner";
    import { Trash2, Save, Star, CheckCircle, Calendar as CalendarIcon, Clock, GitMerge, AlertTriangle } from "lucide-svelte";
    import { Spinner } from "$lib/components/ui/spinner";
    import { onMount } from "svelte";
    import {
        CalendarDate,
        DateFormatter,
        getLocalTimeZone,
        parseDate
    } from "@internationalized/date";
    import { cn } from "@/utils";
    import { i18n } from "@/stores/i18n.svelte.js";
    import type { CoreError } from "@/api/client";

    import { listStore } from "@/app/list.svelte.js";
    import ResponsiveSelect from "@/components/ResponsiveSelect.svelte";
    import SmartImage from "@/components/SmartImage.svelte";
    import {appConfig} from "@/stores/config.svelte";
    import type {ScoreFormat} from "@/api/config/types";

    let {
        open = $bindable(false),
        cid,
        title = i18n.t('list.modal.default_title'),
        contentType = "anime",
        coverImage = "",
        headers
    }: {
        open: boolean;
        cid: string;
        title?: string;
        contentType?: string;
        coverImage?: string;
        headers?: any
    } = $props();

    const TRACKER_META: Record<string, { icon: string; label: string }> = {
        anilist:     { icon: "https://anilist.co/img/icons/favicon-32x32.png",         label: "AniList" },
        myanimelist: { icon: "https://myanimelist.net/favicon.ico",                    label: "MyAnimeList" },
        mal:         { icon: "https://myanimelist.net/favicon.ico",                    label: "MAL" },
        kitsu:       { icon: "https://kitsu.app/favicon.ico",                          label: "Kitsu" },
        simkl:       { icon: "https://simkl.com/favicon.ico",                          label: "Simkl" },
    };

    let scoreFormat = $derived(appConfig.data?.content?.scoreFormat ?? 'point10' as ScoreFormat);

    // Convert 0-10 stored score → display value for the input
    function toDisplayScore(raw: number | string): number | string {
        if (raw === "" || raw === null || raw === undefined) return "";
        const n = typeof raw === "string" ? parseFloat(raw) : raw;
        if (isNaN(n)) return "";
        switch (scoreFormat) {
            case 'point100':      return Math.round(n * 10);
            case 'point5Stars':   return Math.round(n / 2);
            case 'point10Decimal': return n;
            case 'point10':
            default:              return Math.round(n);
        }
    }

    // Convert display value → 0-10 for backend
    function toStoredScore(display: number | string): number | undefined {
        if (display === "" || display === null || display === undefined) return undefined;
        const n = typeof display === "string" ? parseFloat(display) : display;
        if (isNaN(n)) return undefined;
        switch (scoreFormat) {
            case 'point100':      return n / 10;
            case 'point5Stars':   return n * 2;
            case 'point10Decimal':
            case 'point10':
            default:              return n;
        }
    }

    function scoreMax(): number {
        switch (scoreFormat) {
            case 'point100':    return 100;
            case 'point5Stars': return 5;
            default:            return 10;
        }
    }

    function scoreStep(): number {
        return scoreFormat === 'point10Decimal' ? 0.1 : 1;
    }

    function trackerIcon(tracker: string): string {
        return TRACKER_META[tracker.toLowerCase()]?.icon ?? "";
    }
    function trackerLabel(tracker: string): string {
        return TRACKER_META[tracker.toLowerCase()]?.label ?? tracker;
    }

    const COMPARE_FIELDS = [
        { key: "status",      label: "Status" },
        { key: "progress",    label: "Progress" },
        { key: "score",       label: "Score" },
        { key: "startDate",   label: "Start Date" },
        { key: "endDate",     label: "End Date" },
        { key: "repeatCount", label: "Rewatches" },
    ];

    const df = $derived(new DateFormatter(i18n.locale === 'es' ? 'es-ES' : 'en-US', { dateStyle: "long" }));

    let loading = $state(true);
    let submitting = $state(false);
    let isNew = $state(true);
    let totalUnits = $state<number | null>(null);

    let status = $state<ListStatus>("PLANNING");
    let progress = $state<number>(0);
    let score = $state<number | string>("");
    let repeatCount = $state<number>(0);
    let notes = $state<string>("");
    let isPrivate = $state<boolean>(false);

    let startValue = $state<CalendarDate | undefined>();
    let endValue = $state<CalendarDate | undefined>();
    let deleteDialogOpen = $state(false);

    // sources & history
    let sources = $state<EntrySource[]>([]);
    let history = $state<ListEntryChange[]>([]);
    let historyLoading = $state(false);
    let showHistory = $state(false);
    let showSources = $state(false);

    let isTouchDevice = $state(false);

    onMount(() => {
        isTouchDevice = window.matchMedia('(pointer: coarse)').matches;
    });

    function handleStartNativeChange(e: Event) {
        const val = (e.currentTarget as HTMLInputElement).value;
        startValue = val ? parseDate(val) : undefined;
    }

    function handleEndNativeChange(e: Event) {
        const val = (e.currentTarget as HTMLInputElement).value;
        endValue = val ? parseDate(val) : undefined;
    }

    let isAnime = $derived(contentType === "anime");
    let progressLabel = $derived(isAnime ? i18n.t('list.modal.episodes') : i18n.t('list.modal.chapters'));

    let statusOptions = $derived([
        { value: "CURRENT",   label: isAnime ? i18n.t('list.modal.watching') : i18n.t('list.modal.reading') },
        { value: "COMPLETED", label: i18n.t('list.completed') },
        { value: "PLANNING",  label: i18n.t('list.planning') },
        { value: "PAUSED",    label: i18n.t('list.paused') },
        { value: "DROPPED",   label: i18n.t('list.dropped') },
        { value: "REPEATING", label: i18n.t('list.repeating') }
    ]);

    // Whether any field differs between remotes — used to show conflict indicator
    let hasConflicts = $derived(
        sources.length > 1 && COMPARE_FIELDS.some(f => {
            const vals = sources.map(s => String(s[f.key] ?? "")).filter(v => v !== "");
            return vals.length > 1 && new Set(vals).size > 1;
        })
    );

    $effect(() => {
        if (open && cid) {
            loadEntry();
        } else if (!open) {
            resetForm();
        }
    });

    async function loadEntry() {
        loading = true;
        showHistory = false;
        showSources = false;
        history = [];
        sources = [];

        try {
            const existing = listStore.entries.find(e => e.cid === cid);
            if (existing) {
                isNew = false;
                status = existing.status;
                progress = existing.progress;
                score = existing.score != null ? toDisplayScore(existing.score) : "";
                startValue = existing.startDate ? parseDate(existing.startDate.split('T')[0]) : undefined;
                endValue = existing.endDate ? parseDate(existing.endDate.split('T')[0]) : undefined;
                repeatCount = existing.repeatCount ?? 0;
                notes = existing.notes || "";
                isPrivate = existing.isPrivate;
                totalUnits = existing.totalUnits ?? null;
                sources = existing.sources ?? [];

                historyLoading = true;
                listApi.getEntryHistory(cid)
                    .then(r => { history = r.changes; })
                    .catch(() => {})
                    .finally(() => { historyLoading = false; });
            } else {
                isNew = true;
                resetForm();
            }
        } finally {
            loading = false;
        }
    }

    function resetForm() {
        status = "PLANNING";
        progress = 0;
        score = "";
        startValue = undefined;
        endValue = undefined;
        repeatCount = 0;
        notes = "";
        isPrivate = false;
        sources = [];
        history = [];
        showHistory = false;
        showSources = false;
    }

    async function handleSubmit(e: Event) {
        e.preventDefault();
        submitting = true;

        try {
            const body: UpsertEntryBody = {
                cid,
                status,
                progress: progress || 0,
                score: typeof score === 'number' ? score : (parseFloat(score) || undefined),
                startDate: startValue?.toString(),
                endDate: endValue?.toString(),
                repeatCount: repeatCount || 0,
                notes: notes.trim() || undefined,
                isPrivate
            };

            await listApi.upsert(body);
            toast.success(isNew ? i18n.t('list.modal.added_to_list') : i18n.t('list.modal.entry_updated'));

            const existing = listStore.entries.find(e => e.cid === cid);
            const now = new Date().toISOString();
            const updated: EnrichedListEntry = {
                userId:             existing?.userId             ?? 0,
                createdAt:          existing?.createdAt          ?? now,
                trackerIds:         existing?.trackerIds         ?? {},
                externalIds:        existing?.externalIds        ?? null,
                hasExtensionSource: existing?.hasExtensionSource ?? false,
                nsfw:               existing?.nsfw               ?? false,
                totalUnits:         existing?.totalUnits         ?? null,
                titleI18n:          existing?.titleI18n          ?? {},
                sources:            existing?.sources            ?? [],
                cid,
                title,
                contentType,
                coverImage:    coverImage || null,
                updatedAt:     now,
                status,
                progress:      body.progress    ?? 0,
                score: toStoredScore(score),
                startDate:     body.startDate   ?? null,
                endDate:       body.endDate     ?? null,
                repeatCount:   body.repeatCount ?? 0,
                notes:         body.notes       ?? null,
                isPrivate:     body.isPrivate   ?? false,
            };
            listStore.upsertLocal(body, updated);
            open = false;
        } catch (err) {
            const error = err as CoreError;
            toast.error(i18n.t(error.key));
        } finally {
            submitting = false;
        }
    }

    async function handleDelete() {
        submitting = true;
        try {
            await listApi.delete(cid);
            listStore.deleteLocal(cid);
            toast.success(i18n.t('list.modal.removed'));
            open = false;
        } catch (err) {
            const error = err as CoreError;
            toast.error(i18n.t(error.key));
        } finally {
            submitting = false;
            deleteDialogOpen = false;
        }
    }

    function formatChangeField(field: string): string {
        return field.replace(/_/g, ' ');
    }

    function formatChangeDate(dateStr: string): string {
        return new Date(dateStr).toLocaleDateString(i18n.locale === 'es' ? 'es-ES' : 'en-US', {
            month: 'short', day: 'numeric', year: 'numeric'
        });
    }

    function localValueFor(field: string): string {
        switch (field) {
            case "status":      return status;
            case "progress":    return String(progress);
            case "score": return score !== "" ? String(toStoredScore(score) ?? "—") : "—";
            case "startDate":   return startValue?.toString() ?? "—";
            case "endDate":     return endValue?.toString() ?? "—";
            case "repeatCount": return String(repeatCount);
            default:            return "—";
        }
    }

    // Whether a specific field has differing values across remotes
    function fieldHasConflict(field: string): boolean {
        const vals = sources.map(s => String((s as any)[field] ?? "").toUpperCase()).filter(v => v !== "");
        return vals.length > 1 && new Set(vals).size > 1;
    }
</script>

<Dialog.Root bind:open={open}>
    <Dialog.Content class="w-[95vw] sm:max-w-2xl lg:max-w-3xl bg-background border-border p-0 overflow-hidden sm:rounded-sm shadow-lg z-[100] flex flex-col max-h-[95dvh] sm:max-h-[90dvh]">

        {#if loading}
            <div class="h-64 flex flex-col items-center justify-center gap-4 text-muted-foreground">
                <Spinner class="h-8 w-8 animate-spin text-primary" />
                <p class="font-bold">{i18n.t('list.modal.loading')}</p>
            </div>
        {:else}
            <div class="relative z-10 p-6 pb-1 flex items-center gap-5 w-full">
                {#if coverImage}
                    <div class="aspect-auto w-30 object-cover">
                        <SmartImage
                                src={coverImage}
                                alt={title}
                                imageHeaders={headers}
                                class="md:w-20 md:h-28 rounded-sm shadow-lg border border-border/50 hidden sm:block"
                        />
                    </div>
                {/if}
                <div class="min-w-0">
                    <h2 class="text-xl md:text-2xl font-black text-foreground line-clamp-2 leading-tight drop-shadow-md tracking-tight">{title}</h2>
                    <p class="text-sm text-muted-foreground font-bold mt-1.5 uppercase tracking-wider">{isNew ? i18n.t('list.add_to_list') : i18n.t('list.modal.edit')}</p>

                    {#if sources.length > 0}
                        <div class="flex items-center gap-2 mt-2 flex-wrap">
                            {#each sources as source}
                                {@const icon = trackerIcon(source.tracker)}
                                {@const label = trackerLabel(source.tracker)}
                                <div class="inline-flex items-center gap-1.5 px-2 py-1 rounded-sm bg-black/30 border border-white/10">
                                    {#if icon}
                                        <img src={icon} alt={label} class="w-5.5 h-5.5 rounded-sm" />
                                    {/if}
                                    {#if source.syncedAt}
                                        <span class="text-[10px] text-white">· {formatChangeDate(source.syncedAt)}</span>
                                    {/if}
                                </div>
                            {/each}
                            {#if hasConflicts}
                                <div class="inline-flex items-center gap-1 px-2 py-1 rounded-md bg-yellow-500/15 border border-yellow-500/30">
                                    <AlertTriangle class="w-3 h-3 text-yellow-500" />
                                    <span class="text-[11px] font-bold text-yellow-500">Conflicts</span>
                                </div>
                            {/if}
                        </div>
                    {/if}
                </div>
            </div>

            <form onsubmit={handleSubmit} class="flex-1 p-6 pt-4 space-y-6 overflow-y-auto hide-scrollbar">
                <div class="grid grid-cols-2 gap-5">
                    <div class="col-span-2 space-y-2">
                        <Label for="status" class="font-bold text-foreground/90">{i18n.t('list.modal.status')}</Label>
                        <ResponsiveSelect
                                bind:value={status}
                                items={statusOptions}
                                label={i18n.t('list.modal.status')}
                        />
                    </div>

                    <div class="col-span-1 space-y-2">
                        <Label for="score" class="font-bold text-foreground/90">{i18n.t('list.modal.score')}</Label>
                        {#if scoreFormat === 'point5Stars'}
                            <div class="flex items-center gap-1 h-11">
                                {#each [1, 2, 3, 4, 5] as star}
                                    <button
                                            type="button"
                                            class="text-2xl transition-colors {Number(score) >= star ? 'text-amber-400' : 'text-muted-foreground/30'} hover:text-amber-300"
                                            onclick={() => score = score === star ? "" : star}
                                    >★</button>
                                {/each}
                                {#if score !== ""}
                                    <button type="button" class="text-xs text-muted-foreground ml-1 hover:text-foreground" onclick={() => score = ""}>✕</button>
                                {/if}
                            </div>
                        {:else}
                            <div class="relative flex items-center">
                                <Star class="absolute left-3.5 h-4 w-4 text-muted-foreground" />
                                <Input
                                        id="score"
                                        type="number"
                                        step={scoreStep()}
                                        min="0"
                                        max={scoreMax()}
                                        bind:value={score}
                                        class="pl-10 h-11 rounded-sm bg-muted/10 border-border/50 focus-visible:ring-1 focus-visible:ring-primary/50 font-semibold"
                                />
                            </div>
                        {/if}
                    </div>

                    <div class="col-span-1 space-y-2">
                        <Label for="progress" class="font-bold text-foreground/90 truncate">{progressLabel}</Label>
                        <div class="relative flex items-center">
                            <CheckCircle class="absolute left-3.5 h-4 w-4 text-muted-foreground" />
                            <Input id="progress" type="number" min="0" bind:value={progress} class="pl-10 h-11 rounded-sm bg-muted/10 border-border/50 focus-visible:ring-1 focus-visible:ring-primary/50 font-semibold" />
                        </div>
                    </div>
                </div>

                <div class="grid grid-cols-2 gap-5">
                    <div class="col-span-1 flex flex-col gap-2">
                        <Label class="font-bold text-foreground/90 px-1 truncate">{i18n.t('list.modal.start_date')}</Label>
                        {#if isTouchDevice}
                            <div class="relative flex items-center">
                                <CalendarIcon class="absolute left-3.5 h-4 w-4 text-muted-foreground pointer-events-none" />
                                <Input type="date" value={startValue ? startValue.toString() : ""} onchange={handleStartNativeChange} class={cn("pl-10 h-11 w-full font-semibold rounded-sm bg-muted/10 border-border/50 text-xs", !startValue && "text-muted-foreground font-medium")} />
                            </div>
                        {:else}
                            <Popover.Root>
                                <Popover.Trigger>
                                    {#snippet child({ props })}
                                        <Button variant="outline" class={cn("w-full justify-start text-left font-semibold h-11 rounded-sm bg-muted/10 border-border/50 hover:bg-muted/20 px-2", !startValue && "text-muted-foreground font-medium")} {...props}>
                                            <CalendarIcon class="mr-1 h-4 w-4 shrink-0" />
                                            <span class="truncate text-xs">{startValue ? df.format(startValue.toDate(getLocalTimeZone())) : i18n.t('list.modal.select_date')}</span>
                                        </Button>
                                    {/snippet}
                                </Popover.Trigger>
                                <Popover.Content class="w-auto p-0 rounded-sm z-[110]" align="start">
                                    <Calendar type="single" bind:value={startValue} initialFocus captionLayout="dropdown" />
                                </Popover.Content>
                            </Popover.Root>
                        {/if}
                    </div>

                    <div class="col-span-1 flex flex-col gap-2">
                        <Label class="font-bold text-foreground/90 px-1 truncate">{i18n.t('list.modal.end_date')}</Label>
                        {#if isTouchDevice}
                            <div class="relative flex items-center">
                                <CalendarIcon class="absolute left-3.5 h-4 w-4 text-muted-foreground pointer-events-none" />
                                <Input type="date" value={endValue ? endValue.toString() : ""} onchange={handleEndNativeChange} class={cn("pl-10 h-11 w-full font-semibold rounded-sm bg-muted/10 border-border/50 text-xs", !endValue && "text-muted-foreground font-medium")} />
                            </div>
                        {:else}
                            <Popover.Root>
                                <Popover.Trigger>
                                    {#snippet child({ props })}
                                        <Button variant="outline" class={cn("w-full justify-start text-left font-semibold h-11 rounded-sm bg-muted/10 border-border/50 hover:bg-muted/20 px-2", !endValue && "text-muted-foreground font-medium")} {...props}>
                                            <CalendarIcon class="mr-1 h-4 w-4 shrink-0" />
                                            <span class="truncate text-xs">{endValue ? df.format(endValue.toDate(getLocalTimeZone())) : i18n.t('list.modal.select_date')}</span>
                                        </Button>
                                    {/snippet}
                                </Popover.Trigger>
                                <Popover.Content class="w-auto p-0 rounded-sm z-[110]" align="start">
                                    <Calendar type="single" bind:value={endValue} initialFocus captionLayout="dropdown" />
                                </Popover.Content>
                            </Popover.Root>
                        {/if}
                    </div>
                </div>

                <div class="grid grid-cols-2 gap-5">
                    <div class="col-span-1 space-y-2">
                        <Label for="repeat" class="font-bold text-foreground/90 truncate">{isAnime ? i18n.t('list.modal.times_rewatched') : i18n.t('list.modal.times_reread')}</Label>
                        <Input id="repeat" type="number" min="0" bind:value={repeatCount} class="h-11 rounded-sm bg-muted/10 border-border/50 focus-visible:ring-1 focus-visible:ring-primary/50 font-semibold" />
                    </div>

                    <div class="col-span-1 flex items-end">
                        <div class="flex items-center space-x-3 bg-muted/10 p-3 rounded-sm border border-border/50 w-full h-11">
                            <Checkbox id="isPrivate" bind:checked={isPrivate} />
                            <Label for="isPrivate" class="font-bold cursor-pointer text-sm truncate">{i18n.t('list.modal.private')}</Label>
                        </div>
                    </div>
                </div>

                <div class="space-y-2">
                    <Label for="notes" class="font-bold text-foreground/90">{i18n.t('list.modal.notes')}</Label>
                    <Textarea id="notes" bind:value={notes} class="min-h-[100px] rounded-sm bg-muted/10 border-border/50 focus-visible:ring-1 focus-visible:ring-primary/50 font-medium resize-none" />
                </div>

                {#if !isNew && sources.length > 1}
                    <div class="border-t border-border/40 pt-4 space-y-2">
                        <button
                                type="button"
                                class="flex items-center gap-2 text-xs font-bold text-muted-foreground hover:text-foreground transition-colors w-full"
                                onclick={() => showSources = !showSources}
                        >
                            <GitMerge class="h-3.5 w-3.5 shrink-0" />
                            <span>{i18n.t("list.modal.merge")}</span>
                            {#if hasConflicts}
                                <span class="inline-flex items-center gap-0.5 text-[10px] font-bold text-yellow-500">
                                    <AlertTriangle class="w-3 h-3" /> {i18n.t("list.modal.conflict")}
                                </span>
                            {/if}
                            <span class="ml-auto opacity-50 text-[10px]">{showSources ? '▲' : '▼'}</span>
                        </button>

                        {#if showSources}
                            <div class="rounded-md border border-border/40 overflow-hidden text-xs">
                                <div class="grid bg-muted/20 border-b border-border/40 font-bold text-muted-foreground"
                                     style="grid-template-columns: 90px repeat({sources.length + 1}, 1fr)">
                                    <div class="px-3 py-2">Field</div>
                                    <div class="px-3 py-2 flex items-center gap-1.5">
                                        <span class="w-2 h-2 rounded-full bg-primary inline-block"></span>
                                        Merged
                                    </div>
                                    {#each sources as source}
                                        {@const icon = trackerIcon(source.tracker)}
                                        {@const label = trackerLabel(source.tracker)}
                                        <div class="px-3 py-2 flex items-center gap-1.5">
                                            {#if icon}
                                                <img src={icon} alt={label} class="w-3.5 h-3.5 rounded-sm shrink-0" />
                                            {/if}
                                            <span class="truncate">{label}</span>
                                        </div>
                                    {/each}
                                </div>

                                {#each COMPARE_FIELDS as field}
                                    {@const conflict = fieldHasConflict(field.key)}
                                    <div class="grid border-b border-border/30 last:border-0 {conflict ? 'bg-yellow-500/5' : ''}"
                                         style="grid-template-columns: 90px repeat({sources.length + 1}, 1fr)">
                                        <div class="px-3 py-2 font-bold text-muted-foreground flex items-center gap-1">
                                            {#if conflict}
                                                <AlertTriangle class="w-3 h-3 text-yellow-500 shrink-0" />
                                            {/if}
                                            {field.label}
                                        </div>
                                        <div class="px-3 py-2 font-semibold text-foreground">
                                            {localValueFor(field.key)}
                                        </div>
                                        {#each sources as source}
                                            {@const val = source[field.key]}
                                            {@const displayVal = val != null ? String(val) : "—"}
                                            {@const isWinner = String(val ?? "").toUpperCase() === localValueFor(field.key).toUpperCase()}
                                            <div class="px-3 py-2 {isWinner ? 'text-foreground' : 'text-muted-foreground'}">
                                                {displayVal}
                                                {#if isWinner && conflict}
                                                    <span class="text-[9px] text-primary font-bold ml-1 uppercase">used</span>
                                                {/if}
                                            </div>
                                        {/each}
                                    </div>
                                {/each}
                            </div>
                        {/if}
                    </div>
                {/if}

                {#if !isNew}
                    <div class="border-t border-border/40 pt-4 space-y-2">
                        <button
                                type="button"
                                class="flex items-center gap-2 text-xs font-bold text-muted-foreground hover:text-foreground transition-colors w-full"
                                onclick={() => showHistory = !showHistory}
                        >
                            <Clock class="h-3.5 w-3.5 shrink-0" />
                            <span>{i18n.t('list.modal.history')}</span>
                            <span class="ml-auto opacity-50 text-[10px]">{showHistory ? '▲' : '▼'}</span>
                        </button>

                        {#if showHistory}
                            {#if historyLoading}
                                <div class="flex justify-center py-4">
                                    <Spinner class="h-4 w-4 animate-spin text-muted-foreground" />
                                </div>
                            {:else if history.length === 0}
                                <p class="text-xs text-muted-foreground text-center py-3">{i18n.t('list.modal.no_history')}</p>
                            {:else}
                                <div class="space-y-0.5 max-h-48 overflow-y-auto hide-scrollbar">
                                    {#each history as change}
                                        <div class="flex items-center justify-between gap-3 px-2 py-1.5 rounded-md hover:bg-muted/20 text-xs">
                                            <div class="flex items-center gap-2 min-w-0">
                                                <span class="font-bold text-foreground/60 capitalize shrink-0">{formatChangeField(change.field)}</span>
                                                {#if change.oldValue}
                                                    <span class="text-muted-foreground line-through truncate max-w-[60px]">{change.oldValue}</span>
                                                    <span class="text-muted-foreground shrink-0">→</span>
                                                {/if}
                                                <span class="text-foreground font-semibold truncate">{change.newValue}</span>
                                            </div>
                                            <div class="flex items-center gap-1.5 shrink-0 text-muted-foreground/50 text-[10px]">
                                                {#if change.tracker}
                                                    {@const icon = trackerIcon(change.tracker)}
                                                    {#if icon}
                                                        <img src={icon} alt={change.tracker} class="w-3 h-3 rounded-sm opacity-60" />
                                                    {:else}
                                                        <span class="uppercase font-bold text-primary/60">{change.tracker}</span>
                                                    {/if}
                                                    <span>·</span>
                                                {/if}
                                                <span>{formatChangeDate(change.changedAt)}</span>
                                            </div>
                                        </div>
                                    {/each}
                                </div>
                            {/if}
                        {/if}
                    </div>
                {/if}
            </form>

            <Dialog.Footer class="p-5 border-t border-border bg-muted/10">
                <div class="flex items-center justify-between w-full gap-3">
                    <div class="flex shrink-0">
                        {#if !isNew}
                            <Button type="button" variant="destructive" size="icon" class="h-11 w-11 rounded-sm shadow-sm" onclick={handleDelete} disabled={submitting}>
                                <Trash2 class="h-5 w-5" />
                            </Button>
                        {:else}
                            <div class="w-0 sm:w-11"></div>
                        {/if}
                    </div>

                    <div class="flex items-center gap-3 flex-1 sm:flex-initial justify-end">
                        <Button
                                type="button"
                                variant="outline"
                                class="flex-1 sm:w-32 h-11 rounded-sm font-bold border-border/50 hover:bg-muted/20"
                                disabled={submitting}
                                onclick={() => open = false}
                        >
                            {i18n.t('list.modal.cancel')}
                        </Button>
                        <Button
                                type="submit"
                                onclick={handleSubmit}
                                class="flex-1 sm:w-32 h-11 rounded-sm font-bold shadow-sm"
                                disabled={submitting}
                        >
                            {#if submitting}
                                <Spinner class="h-4 w-4 mr-2 animate-spin" />
                                <span class="truncate">{i18n.t('list.modal.saving')}</span>
                            {:else}
                                <Save class="h-4 w-4 mr-2" />
                                <span class="truncate">{isNew ? i18n.t('list.modal.save') : i18n.t('list.modal.update')}</span>
                            {/if}
                        </Button>
                    </div>
                </div>
            </Dialog.Footer>
        {/if}
    </Dialog.Content>
</Dialog.Root>

<style>
    :global([data-dialog-close]) {
        display: none !important;
    }

    :global(input[type="date"]::-webkit-calendar-picker-indicator) {
        background: transparent;
        bottom: 0;
        color: transparent;
        cursor: pointer;
        height: auto;
        left: 0;
        position: absolute;
        right: 0;
        top: 0;
        width: auto;
    }
</style>