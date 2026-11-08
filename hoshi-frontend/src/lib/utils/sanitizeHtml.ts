const ALLOWED_TAGS = new Set([
    "p", "br", "hr", "div", "span", "section", "article", "header", "footer",
    "h1", "h2", "h3", "h4", "h5", "h6",
    "b", "strong", "i", "em", "u", "s", "del", "ins", "mark", "small", "sub", "sup",
    "blockquote", "q", "cite", "pre", "code",
    "ul", "ol", "li", "dl", "dt", "dd",
    "table", "thead", "tbody", "tfoot", "tr", "th", "td", "caption",
    "figure", "figcaption", "img", "a", "ruby", "rt", "rp",
]);

/** Elements removed together with everything inside them. */
const DROP_WITH_CONTENT = new Set([
    "script", "style", "iframe", "frame", "frameset", "object", "embed", "applet",
    "noscript", "template", "svg", "math", "form", "input", "button", "textarea",
    "select", "option", "link", "meta", "base", "head", "title", "audio", "video",
    "source", "track", "canvas", "dialog",
]);

const ALLOWED_ATTRS: Record<string, Set<string>> = {
    "*": new Set(["title", "lang", "dir"]),
    a: new Set(["href"]),
    img: new Set(["src", "alt", "width", "height"]),
    td: new Set(["colspan", "rowspan"]),
    th: new Set(["colspan", "rowspan"]),
    ol: new Set(["start"]),
};

const URL_ATTRS = new Set(["href", "src"]);

function isSafeUrl(value: string, attr: string): boolean {
    const v = value.trim().replace(/[\u0000- ]/g, "").toLowerCase();
    if (v.startsWith("http:") || v.startsWith("https:")) return true;
    // Inline images are fine as images, never as links.
    if (attr === "src" && /^data:image\/(png|jpe?g|gif|webp|avif);/.test(v)) return true;
    // Relative URLs and fragments carry no scheme.
    return !/^[a-z][a-z0-9+.-]*:/.test(v);
}

function cleanNode(node: Node): void {
    for (const child of Array.from(node.childNodes)) {
        if (child.nodeType === Node.TEXT_NODE) continue;
        if (child.nodeType !== Node.ELEMENT_NODE) {
            child.remove(); // comments, processing instructions, ...
            continue;
        }

        const el = child as Element;
        const tag = el.tagName.toLowerCase();

        if (DROP_WITH_CONTENT.has(tag)) {
            el.remove();
            continue;
        }

        cleanNode(el); // children first, so unwrapping keeps already-clean content

        if (!ALLOWED_TAGS.has(tag)) {
            el.replaceWith(...Array.from(el.childNodes));
            continue;
        }

        const allowed = ALLOWED_ATTRS[tag];
        for (const attr of Array.from(el.attributes)) {
            const name = attr.name.toLowerCase();
            const ok =
                (ALLOWED_ATTRS["*"].has(name) || allowed?.has(name)) &&
                (!URL_ATTRS.has(name) || isSafeUrl(attr.value, name));
            if (!ok) el.removeAttribute(attr.name);
        }

        if (tag === "a") {
            el.setAttribute("rel", "noopener noreferrer nofollow");
            el.setAttribute("target", "_blank");
        }
    }
}

export function sanitizeHtml(dirty: string): string {
    if (!dirty) return "";
    const doc = new DOMParser().parseFromString(dirty, "text/html");
    cleanNode(doc.body);
    return doc.body.innerHTML;
}
