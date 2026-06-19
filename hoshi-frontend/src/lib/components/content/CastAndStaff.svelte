<script lang="ts">
    import type { Character, StaffMember } from "$lib/api/content/types";
    import { Mic2, User, ChevronDown, ChevronUp } from "lucide-svelte";
    import { i18n } from "@/stores/i18n.svelte.js";
    import { Button } from "$lib/components/ui/button";

    const formatRole = (role: string | undefined | null) => {
        if (!role) return '';
        const normalized = role.toUpperCase().replace(/\s+/g, '_');
        const key = `roles.${normalized}` as any;
        const translated = i18n.t(key);

        return translated === key ? role : translated;
    };

    let { characters, staff }: { characters: Character[], staff: StaffMember[] } = $props();

    const CHAR_LIMIT = 12;
    const STAFF_LIMIT = 12;

    let showAllChars = $state(false);
    let showAllStaff = $state(false);

    const visibleCharacters = $derived(showAllChars ? characters : characters.slice(0, CHAR_LIMIT));
    const visibleStaff = $derived(showAllStaff ? staff : staff.slice(0, STAFF_LIMIT));
</script>

<div class="space-y-8">
    {#if characters && characters.length > 0}
        <section class="space-y-3">
            <h3 class="text-base md:text-lg font-bold tracking-tight">
                {i18n.t('content.characters')}
            </h3>

            <div class="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-6 lg:grid-cols-8 gap-3">
                {#each visibleCharacters as char}
                    <div class="group relative rounded-sm overflow-hidden border border-border/20 bg-card hover:border-border/50 transition-colors">
                        <div class="aspect-[3/4] w-full overflow-hidden bg-muted/20">
                            {#if char.image}
                                <img
                                        src={char.image}
                                        alt={char.name}
                                        class="w-full h-full object-cover transition-transform duration-500 group-hover:scale-105"
                                />
                            {:else}
                                <div class="w-full h-full flex items-center justify-center">
                                    <User class="h-6 w-6 text-muted-foreground/30" />
                                </div>
                            {/if}
                        </div>

                        <div class="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/90 via-black/55 to-transparent pt-8 p-2">
                            <p class="font-semibold text-[11px] text-white leading-tight truncate">{char.name}</p>
                            <p class="text-[9px] text-white/60 capitalize truncate">{formatRole(char.role)}</p>
                            {#if char.actor}
                                <div class="mt-0.5 flex items-center gap-1 text-[9px] font-medium text-primary">
                                    <Mic2 class="h-2 w-2 shrink-0" />
                                    <span class="truncate">{char.actor}</span>
                                </div>
                            {/if}
                        </div>
                    </div>
                {/each}
            </div>

            {#if characters.length > CHAR_LIMIT}
                <div class="flex justify-center pt-1">
                    <Button
                            variant="outline"
                            size="sm"
                            class="rounded-sm px-5 text-xs font-semibold bg-muted/20 hover:bg-muted/50 transition-colors"
                            onclick={() => showAllChars = !showAllChars}
                    >
                        {#if showAllChars}
                            <ChevronUp class="w-3.5 h-3.5 mr-1.5" />
                        {:else}
                            <ChevronDown class="w-3.5 h-3.5 mr-1.5" />
                            ({characters.length - CHAR_LIMIT})
                        {/if}
                    </Button>
                </div>
            {/if}
        </section>
    {/if}
</div>