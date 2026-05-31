pub struct OscBundle {
    pub name: &'static str,
    pub files: &'static [OscFile],
}

pub struct OscFile {
    pub url: &'static str,
    pub dest: OscDest, // Scripts, ScriptOpts, Fonts
}

pub enum OscDest {
    Scripts,
    ScriptOpts,
    Fonts,
}

pub static OSC_REGISTRY: &[OscBundle] = &[
    OscBundle {
        name: "modernz",
        files: &[
            OscFile { url: "https://github.com/Samillion/ModernZ/releases/latest/download/modernz.lua", dest: OscDest::Scripts },
            OscFile { url: "https://github.com/Samillion/ModernZ/releases/latest/download/modernz.conf", dest: OscDest::ScriptOpts },
            OscFile { url: "https://github.com/Samillion/ModernZ/releases/latest/download/modernz-icons.ttf", dest: OscDest::Fonts },
        ],
    },
    OscBundle {
        name: "mpv-osc-modern",
        files: &[
            OscFile { url: "https://github.com/maoiscat/mpv-osc-modern/raw/refs/heads/main/modern.lua", dest: OscDest::Scripts },
            OscFile { url: "https://github.com/maoiscat/mpv-osc-modern/raw/refs/heads/main/Material-Design-Iconic-Font.ttf", dest: OscDest::Fonts },
        ],
    },
    OscBundle {
        name: "hayase-osc",
        files: &[
            OscFile { url: "https://github.com/nekoxuee/hayase-osc/raw/refs/heads/main/scripts/hayase-osc.lua", dest: OscDest::Scripts },
            OscFile { url: "https://github.com/nekoxuee/hayase-osc/raw/refs/heads/main/fonts/Lucide.ttf", dest: OscDest::Fonts },
        ],
    },
];

pub struct KnownScript {
    pub name: &'static str,
    pub description: &'static str,
    pub repo_url: &'static str,
    pub url: &'static str,
}

pub static SCRIPT_REGISTRY: &[KnownScript] = &[
    KnownScript {
        name: "thumbfast.lua",
        description: "High-performance on-the-fly thumbnailer for the seekbar.",
        repo_url: "https://github.com/po5/thumbfast",
        url: "https://github.com/po5/thumbfast/raw/refs/heads/master/thumbfast.lua",
    },
];