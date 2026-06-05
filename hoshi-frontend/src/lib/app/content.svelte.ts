import { page } from "$app/state";
import { goto } from "$app/navigation";

import { contentApi } from "@/api/content/content";
import { i18n } from "@/stores/i18n.svelte.js";
import {primaryMetadata} from "@/api/content/types";
import type { FullContent } from "@/api/content/types";
import { layoutState } from '@/stores/layout.svelte.js';
import { appConfig } from "@/stores/config.svelte.js";
import { normalizeFullContent, type NormalizedCard } from "@/utils/normalize";
import {extensions} from "@/stores/extensions.svelte";
import {extensionsApi} from "@/api/extensions/extensions";

export type NormalizedRelation = {
    targetCid: string;
    relationType: string;
    card: NormalizedCard;
};

export class ContentDetailState {
    isLoading = $state(true);
    error = $state<any>(null);
    fullContent = $state<FullContent | null>(null);

    synopsisElement = $state<HTMLElement | null>(null);
    canTruncate = $state(false);

    relations = $state<NormalizedRelation[]>([]);
    relationsLoading = $state(false);

    params = $derived(page.params as Record<string, string>);
    pathParts = $derived(this.params.path ? this.params.path.split('/') : []);
    source = $derived(this.pathParts.length === 2 ? this.pathParts[0] : "");
    id = $derived(this.pathParts.length === 2 ? atob(this.pathParts[1]) : "");
    cid = $derived(this.pathParts.length === 1 ? this.pathParts[0] : "");
    headers: Record<string, string> | undefined;

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
            await this.handleResponse(res);
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

            if (extensions.isTachiyomi(this.source) ) {
                try {
                    const metadata = res.metadata.find(
                        item => item.sourceName === this.source
                    );

                    if (metadata) {
                        this.headers = await extensionsApi.getImageRequestHeaders(
                            this.source,
                            metadata.coverImage
                        );
                    }
                } catch (e) {
                    console.warn("Could not fetch tachiyomi image headers", e);
                }
            }
            await this.handleResponse(res);
        } catch (e) {
            this.handleError(e);
        } finally {
            this.isLoading = false;
        }
    }

    private async handleResponse(res: FullContent) {
        this.fullContent = res;

        const meta = primaryMetadata(res, appConfig.data?.content?.preferredMetadataProvider);
        if (meta) {
            const pref = appConfig.data?.ui?.titleLanguage || 'romaji';
            const title = meta.titleI18n?.[pref] || meta.title || '';
            layoutState.title = title;
        }

        const [, ] = await Promise.all([
            this.loadRelations(res.relations),
        ]);
    }

    private async loadRelations(rawRelations: FullContent['relations']) {
        if (!rawRelations?.length) return;

        this.relationsLoading = true;
        try {
            const settled = await Promise.allSettled(
                rawRelations.map(async (relation) => {
                    const content = await contentApi.get_by_cid(relation.targetCid);
                    return {
                        targetCid: relation.targetCid,
                        relationType: relation.relationType,
                        card: normalizeFullContent(content),
                    } satisfies NormalizedRelation;
                })
            );

            this.relations = settled
                .filter(r => r.status === "fulfilled")
                .map(r => (r as PromiseFulfilledResult<NormalizedRelation>).value);
        } finally {
            this.relationsLoading = false;
        }
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