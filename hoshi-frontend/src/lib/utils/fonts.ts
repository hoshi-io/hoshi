// Bundled fonts (all OFL-1.1). Importing only registers @font-face rules: a file is
// loaded the first time its font is used. Scripts they don't cover fall back to system fonts.
import "@fontsource-variable/inter";
import "@fontsource-variable/roboto";
import "@fontsource-variable/nunito";
import "@fontsource-variable/outfit";
import "@fontsource-variable/lexend";
import "@fontsource-variable/atkinson-hyperlegible-next";
import "@fontsource/opendyslexic/400.css";
import "@fontsource/opendyslexic/700.css";
import "@fontsource-variable/literata";
import "@fontsource-variable/merriweather";
import "@fontsource-variable/lora";
import "@fontsource-variable/source-serif-4";
import "@fontsource-variable/jetbrains-mono";

import type { AppFont, FontFamily } from "@/api/config/types";

const SANS = 'ui-sans-serif, system-ui, sans-serif, "Apple Color Emoji", "Segoe UI Emoji", "Noto Color Emoji"';
const SERIF = 'ui-serif, Georgia, Cambria, "Times New Roman", Times, serif';
const MONO = 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", monospace';

export interface FontOption {
    label: string;
    stack: string;
}

export const FONTS: Record<FontFamily | AppFont, FontOption> = {
    system:        { label: "System",          stack: SANS },
    sans:          { label: "Sans",            stack: SANS },
    serif:         { label: "Serif",           stack: SERIF },
    mono:          { label: "Mono",            stack: MONO },
    literata:      { label: "Literata",        stack: `"Literata Variable", ${SERIF}` },
    merriweather:  { label: "Merriweather",    stack: `"Merriweather Variable", ${SERIF}` },
    lora:          { label: "Lora",            stack: `"Lora Variable", ${SERIF}` },
    sourceSerif:   { label: "Source Serif",    stack: `"Source Serif 4 Variable", ${SERIF}` },
    inter:         { label: "Inter",           stack: `"Inter Variable", ${SANS}` },
    roboto:        { label: "Roboto",          stack: `"Roboto Variable", ${SANS}` },
    nunito:        { label: "Nunito",          stack: `"Nunito Variable", ${SANS}` },
    outfit:        { label: "Outfit",          stack: `"Outfit Variable", ${SANS}` },
    lexend:        { label: "Lexend",          stack: `"Lexend Variable", ${SANS}` },
    atkinson:      { label: "Atkinson Hyperlegible", stack: `"Atkinson Hyperlegible Next Variable", ${SANS}` },
    openDyslexic:  { label: "OpenDyslexic",    stack: `"OpenDyslexic", ${SANS}` },
    jetbrainsMono: { label: "JetBrains Mono",  stack: `"JetBrains Mono Variable", ${MONO}` },
};

export const APP_FONTS: AppFont[] = ["system", "inter", "roboto", "nunito", "outfit", "lexend", "atkinson", "openDyslexic"];

export const READER_FONTS: FontFamily[] = [
    "sans", "serif", "mono",
    "literata", "merriweather", "lora", "sourceSerif",
    "inter", "roboto", "nunito", "lexend",
    "atkinson", "openDyslexic", "jetbrainsMono",
];

export function fontStack(id: string | null | undefined): string {
    return FONTS[id as keyof typeof FONTS]?.stack ?? SANS;
}
