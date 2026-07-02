import { page } from "$app/state";
import { goto } from "$app/navigation";

import { contentApi } from "@/api/content/content";
import { i18n } from "@/stores/i18n.svelte.js";
import {primaryMetadata} from "@/api/content/types";
import type { FullContent } from "@/api/content/types";
import { layoutState } from '@/stores/layout.svelte.js';
import { appConfig } from "@/stores/config.svelte.js";
import {type NormalizedCard, type NormalizedRelation, normalizeRelationCard} from "@/utils/normalize";
import {extensions} from "@/stores/extensions.svelte";
import {extensionsApi} from "@/api/extensions/extensions";
import {historyStore} from "@/stores/history.svelte";

export class ContentDetailState {
    isLoading = $state(true);
    error = $state<any>(null);
    fullContent = $state<FullContent | null>(null);

    relations = $state<NormalizedRelation[]>([]);

    params = $derived(page.params as Record<string, string>);
    pathParts = $derived(this.params.path ? this.params.path.split('/') : []);
    source = $derived(this.pathParts.length === 2 ? this.pathParts[0] : "");
    id = $derived(this.pathParts.length === 2 ? atob(this.pathParts[1]) : "");
    cid = $derived(this.pathParts.length === 1 ? this.pathParts[0] : "");
    headers: Record<string, string> | undefined;

    isDerived = $derived(this.fullContent?.trackerMappings.length == 0)

    constructor() {
        $effect(() => {
            if (this.cid) {
                this.loadContentByCid(this.cid);
            } else if (this.source && this.id) {
                this.loadContent(this.source, this.id);
            }
        });
    }

    async loadContentByCid(cid: string) {
        this.isLoading = true;
        this.error = null;
        this.fullContent = null;
        this.relations = [];

        try {
            const res = await contentApi.get_by_cid(cid);
            this.handleResponse(res);
        } catch (e) {
            this.handleError(e);
        } finally {
            this.isLoading = false;
        }
    }

    async loadContent(src: string, entryId: string) {
        this.isLoading = true;
        this.error = null;
        this.fullContent = null;
        this.relations = [];

        try {
            const res = await contentApi.get(src, entryId);

            this.handleResponse(res);
        } catch (e) {
            this.handleError(e);
        } finally {
            this.isLoading = false;
        }
    }

    private handleResponse(res: FullContent) {
        this.fullContent = res;
        console.log(res)

        const meta = primaryMetadata(res, appConfig.data?.content?.preferredMetadataProvider);
        if (meta) {
            const pref = appConfig.data?.ui?.titleLanguage || 'romaji';
            const title = meta.titleI18n?.[pref] || meta.title || '';
            layoutState.title = title;

            historyStore.add({
                id: res.content.cid,
                title,
                coverImage: meta.coverImage ?? null
            });
        }

        if (meta && extensions.isTachiyomi(meta.sourceName) && meta.coverImage) {
            extensionsApi.getImageRequestHeaders(meta.sourceName, meta.coverImage)
                .then(headers => { this.headers = headers; })
                .catch(e => console.warn("Could not fetch tachiyomi image headers", e));
        }

        this.relations = (res.relations ?? []).map((relation) => ({
            targetCid: relation.targetCid,
            targetTrackerName: relation.targetTrackerName,
            targetTrackerId: relation.targetTrackerId,
            relationType: relation.relationType,
            card: normalizeRelationCard(relation),
        }));
    }

    private handleError(e: any) {
        this.error = e;
        console.log(e);
        layoutState.title = i18n.t('errors.error');
    }

    watchNow() {
        if (this.fullContent?.content.contentType === 'anime') {
            goto(`/watch/${this.cid}/1`);
        }
    }

    retry() {
        if (this.cid) {
            this.loadContentByCid(this.cid);
        } else if (this.source && this.id) {
            this.loadContent(this.source, this.id);
        }
    }
}