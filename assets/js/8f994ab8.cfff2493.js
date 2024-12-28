"use strict";(self.webpackChunkwebsite=self.webpackChunkwebsite||[]).push([[54207],{55054:(e,n,s)=>{s.r(n),s.d(n,{assets:()=>d,contentTitle:()=>a,default:()=>x,frontMatter:()=>l,metadata:()=>c,toc:()=>h});var i=s(24246),t=s(71670),r=s(79022),o=s(59702);const l={title:"Non-WASM plugin",sidebar_label:"Non-WASM",toc_max_heading_level:5},a=void 0,c={id:"proto/non-wasm-plugin",title:"Non-WASM plugin",description:"The non-WASM plugin is by design, very simple. It's a JSON, TOML, or YAML file that describes a",source:"@site/docs/proto/non-wasm-plugin.mdx",sourceDirName:"proto",slug:"/proto/non-wasm-plugin",permalink:"/docs/proto/non-wasm-plugin",draft:!1,unlisted:!1,editUrl:"https://github.com/moonrepo/moon/tree/master/website/docs/proto/non-wasm-plugin.mdx",tags:[],version:"current",frontMatter:{title:"Non-WASM plugin",sidebar_label:"Non-WASM",toc_max_heading_level:5},sidebar:"proto",previous:{title:"WASM",permalink:"/docs/proto/wasm-plugin"},next:{title:"activate",permalink:"/docs/proto/commands/activate"}},d={},h=[{value:"Create a plugin",id:"create-a-plugin",level:2},{value:"Platform variations",id:"platform-variations",level:3},{value:"Downloading and installing",id:"downloading-and-installing",level:3},{value:"Executables",id:"executables",level:4},{value:"Global packages",id:"global-packages",level:4},{value:"Resolving versions",id:"resolving-versions",level:3},{value:"Git tags",id:"git-tags",level:4},{value:"JSON manifest",id:"json-manifest",level:4},{value:"Versions and aliases<VersionLabel></VersionLabel>",id:"versions-and-aliases",level:4},{value:"Version patterns",id:"version-patterns",level:4},{value:"Detecting versions",id:"detecting-versions",level:3}];function p(e){const n={a:"a",admonition:"admonition",blockquote:"blockquote",code:"code",em:"em",h2:"h2",h3:"h3",h4:"h4",li:"li",p:"p",ul:"ul",...(0,t.a)(),...e.components};return(0,i.jsxs)(i.Fragment,{children:[(0,i.jsx)(n.p,{children:"The non-WASM plugin is by design, very simple. It's a JSON, TOML, or YAML file that describes a\nschema for the tool, how it should be installed, and how it should be invoked. Since this is a\nstatic configuration file, it does not support any logic or complex behavior, and is merely for\nsimple and common use cases, like CLIs."}),"\n",(0,i.jsx)(n.admonition,{type:"info",children:(0,i.jsx)(n.p,{children:"JSON and YAML support was added in proto v0.42."})}),"\n",(0,i.jsx)(n.h2,{id:"create-a-plugin",children:"Create a plugin"}),"\n",(0,i.jsxs)(n.p,{children:["Let's start by creating a new plugin, and defining the ",(0,i.jsx)(n.code,{children:"name"})," and ",(0,i.jsx)(n.code,{children:"type"})," fields. The type can either\nbe ",(0,i.jsx)(n.code,{children:"language"}),", ",(0,i.jsx)(n.code,{children:"dependency-manager"}),", ",(0,i.jsx)(n.code,{children:"package-manager"}),", or ",(0,i.jsx)(n.code,{children:"cli"}),". For this example, we'll create a\nplugin for our fake product called Protostar, a CLI tool."]}),"\n",(0,i.jsx)(o.Z,{title:"protostar",data:{name:"Protostar",type:"cli"}}),"\n",(0,i.jsx)(n.h3,{id:"platform-variations",children:"Platform variations"}),"\n",(0,i.jsxs)(n.p,{children:["Native tools are often platform specific, and proto supports this by allowing you to define\nvariations based on operating system using the ",(0,i.jsx)(n.code,{children:"[platform]"})," section. For non-native tools, this\nsection can typically be skipped."]}),"\n",(0,i.jsxs)(n.p,{children:["This section requires a mapping of Rust\n",(0,i.jsxs)(n.a,{href:"https://doc.rust-lang.org/std/env/consts/constant.OS.html",children:[(0,i.jsx)(n.code,{children:"OS"})," strings"]})," to platform settings. The\nfollowing settings are available:"]}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"archs"})," - A list of architectures supported for this platform. If not provided, supports all\narchs."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"archive-prefix"})," - If the tool is distributed as an archive (zip, tar, etc), this is the name of\nthe direct folder within the archive that contains the tool, and will be removed when unpacking\nthe archive. If there is no prefix folder within the archive, this setting can be omitted."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"exes-dir"})," - A relative path to a directory that contains pre-installed executables."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"exe-path"})," - The path to the main executable binary within the archive (without the prefix). If\nthe tool is distributed as a single binary, this setting can be typically omitted."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"checksum-file"})," - Name of the checksum file to verify the downloaded file with. If the tool does\nnot support checksum verification, this setting can be omitted."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"download-file"})," (required) - Name of the file to download.\n",(0,i.jsx)(n.a,{href:"#downloading-and-installing",children:"Learn more about downloading"}),"."]}),"\n"]}),"\n",(0,i.jsx)(o.Z,{title:"protostar",data:{platform:{linux:{archivePrefix:"protostar-linux",exePath:"bin/protostar",checksumFile:"protostar-{arch}-unknown-linux-{libc}.sha256",downloadFile:"protostar-{arch}-unknown-linux-{libc}.tar.gz"},macos:{archivePrefix:"protostar-macos",exePath:"bin/protostar",checksumFile:"protostar-{arch}-apple-darwin.sha256",downloadFile:"protostar-{arch}-apple-darwin.tar.xz"},windows:{archivePrefix:"protostar-windows",exePath:"bin/protostar.exe",checksumFile:"protostar-{arch}-pc-windows-msvc.sha256",downloadFile:"protostar-{arch}-pc-windows-msvc.zip"}}}}),"\n",(0,i.jsxs)(n.p,{children:["You may have noticed tokens above, like ",(0,i.jsx)(n.code,{children:"{arch}"}),". These are special tokens that are replaced with a\ndynamic value at runtime, based on the current host machine executing the code. The following tokens\nare available:"]}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"{version}"})," - The currently resolved version, as a fully-qualified semantic or calendar version."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"{versionMajor}"})," / ",(0,i.jsx)(n.code,{children:"{versionYear}"})," - Only the major version. ",(0,i.jsx)(r.Z,{version:"0.41.4"})]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"{versionMajorMinor}"})," / ",(0,i.jsx)(n.code,{children:"{versionYearMonth}"})," - Only the major + minor versions.","\n",(0,i.jsx)(r.Z,{version:"0.41.4"}),"\n"]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"{versionPrerelease}"})," - The prerelease identifier, if applicable. Returns an empty string\notherwise. ",(0,i.jsx)(r.Z,{version:"0.41.4"})]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"{versionBuild}"})," - The build identifier, if applicable. Returns an empty string otherwise.","\n",(0,i.jsx)(r.Z,{version:"0.41.4"}),"\n"]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"{arch}"})," - The architecture of the host machine, like ",(0,i.jsx)(n.code,{children:"x86_64"}),". These values map to Rust's\n",(0,i.jsxs)(n.a,{href:"https://doc.rust-lang.org/std/env/consts/constant.ARCH.html",children:[(0,i.jsx)(n.code,{children:"ARCH"})," constant"]}),", but can be\ncustomized with ",(0,i.jsx)(n.a,{href:"#downloading-and-installing",children:(0,i.jsx)(n.code,{children:"install.arch"})}),"."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"{os}"})," - The operating system of the host machine, like ",(0,i.jsx)(n.code,{children:"windows"}),". These values map to Rust's\n",(0,i.jsxs)(n.a,{href:"https://doc.rust-lang.org/std/env/consts/constant.OS.html",children:[(0,i.jsx)(n.code,{children:"OS"})," constant"]}),"."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"{libc}"})," - For Linux machines, this is the current libc implementation, either ",(0,i.jsx)(n.code,{children:"gnu"})," or ",(0,i.jsx)(n.code,{children:"musl"}),".","\n",(0,i.jsx)(r.Z,{version:"0.31.2"}),"\n"]}),"\n"]}),"\n",(0,i.jsx)(n.h3,{id:"downloading-and-installing",children:"Downloading and installing"}),"\n",(0,i.jsxs)(n.p,{children:["A non-WASM plugin ",(0,i.jsx)(n.em,{children:"only"})," supports downloading pre-built tools, typically as an archive, and does\n",(0,i.jsx)(n.em,{children:"not"})," support building from source. The ",(0,i.jsx)(n.code,{children:"[install]"})," section can be used to configure how the tool\nshould be downloaded and installed into the toolchain. The following settings are available:"]}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"arch"})," - A mapping of Rust\n",(0,i.jsxs)(n.a,{href:"https://doc.rust-lang.org/std/env/consts/constant.ARCH.html",children:[(0,i.jsx)(n.code,{children:"ARCH"})," strings"]})," to custom values for\nthe ",(0,i.jsx)(n.code,{children:"{arch}"})," token. This is useful if the tool has different terminology."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"libc"})," - A mapping of custom values for the ",(0,i.jsx)(n.code,{children:"{libc}"})," token."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"checksum-url"})," - A secure URL to download the checksum file for verification. If the tool does not\nsupport checksum verification, this setting can be omitted."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"checksum-url-canary"})," - A URL for canary releases."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"checksum-public-key"})," - Public key used for verifying checksums. Only used for ",(0,i.jsx)(n.code,{children:".minisig"})," files."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"download-url"})," (required) - A secure URL to download the tool/archive."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"download-url-canary"})," - A URL for canary releases."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"primary"})," - Configures the primary executable."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"secondary"})," - Configures secondary executables."]}),"\n"]}),"\n",(0,i.jsxs)(n.p,{children:["The URL settings support ",(0,i.jsx)(n.code,{children:"{checksum_file}"})," and ",(0,i.jsx)(n.code,{children:"{download_file}"})," tokens, which will be replaced with\nthe values from the ",(0,i.jsx)(n.code,{children:"[platform]"})," section."]}),"\n",(0,i.jsx)(o.Z,{title:"protostar",data:{install:{checksumUrl:"https://github.com/moonrepo/protostar/releases/download/v{version}/{checksum_file}",downloadUrl:"https://github.com/moonrepo/protostar/releases/download/v{version}/{download_file}",arch:{aarch64:"arm64",x86_64:"x64"}}}}),"\n",(0,i.jsx)(n.h4,{id:"executables",children:"Executables"}),"\n",(0,i.jsxs)(n.p,{children:["The available executables (bins and shims) can be customized with the ",(0,i.jsx)(n.code,{children:"[install.exes]"})," section,\nwhich is required. This setting requires a map, where the key is the executable file name, and the\nvalue is an object of the following options:"]}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"exe-path"})," - The file to execute, relative from the tool directory. On Windows, the ",(0,i.jsx)(n.code,{children:".exe"}),"\nextension will automatically be appended. If you need more control over platform variance, use\n",(0,i.jsx)(n.code,{children:"[platform.*.exe-path]"})," instead."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"no-bin"})," - Do not symlink a binary in ",(0,i.jsx)(n.code,{children:"~/.proto/bin"}),"."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"no-shim"}),"- Do not generate a shim in ",(0,i.jsx)(n.code,{children:"~/.proto/shims"}),"."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"parent-exe-name"})," - Name of a parent executable required to execute the executable path. For\nexample, ",(0,i.jsx)(n.code,{children:"node"})," is required for ",(0,i.jsx)(n.code,{children:".js"})," files."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"primary"})," - Is the main executable in the tool. There can only be 1 primary!","\n",(0,i.jsx)(r.Z,{version:"0.42.0"}),"\n"]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"shim-before-args"})," - Custom args to prepend to user-provided args within the generated shim."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"shim-after-args"})," - Custom args to append to user-provided args within the generated shim."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"shim-env-vars"})," - Custom environment variables to set when executing the shim."]}),"\n"]}),"\n",(0,i.jsxs)(n.p,{children:["This field supports both the required primary executable, and optional secondary executables. The\nprimary executable must be marked with ",(0,i.jsx)(n.code,{children:"primary = true"}),"."]}),"\n",(0,i.jsx)(o.Z,{title:"protostar",data:{install:{exes:{protostar:{exePath:"bins/protostar",primary:!0,shimBeforeArgs:["--verbose"]},"protostar-debug":{exePath:"bins/protostar-debug",noShim:!0}}}}}),"\n",(0,i.jsx)(n.h4,{id:"global-packages",children:"Global packages"}),"\n",(0,i.jsxs)(n.p,{children:["The ",(0,i.jsx)(n.code,{children:"[packages]"})," sections can be configured that provides information about where global packages\nare stored."]}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"globals-lookup-dirs"})," - A list of directories where global binaries are stored. This setting\nsupports interpolating environment variables via the syntax ",(0,i.jsx)(n.code,{children:"$ENV_VAR"}),"."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"globals-prefix"})," - A string that all package names are prefixed with. For example, Cargo/Rust\nbinaries are prefixed with ",(0,i.jsx)(n.code,{children:"cargo-"}),"."]}),"\n"]}),"\n",(0,i.jsx)(o.Z,{title:"protostar",data:{packages:{globalsLookupDirs:["$PROTOSTAR_HOME/bin","$HOME/.protostar/bin"]}}}),"\n",(0,i.jsx)(n.h3,{id:"resolving-versions",children:"Resolving versions"}),"\n",(0,i.jsxs)(n.p,{children:["Now that the tool can be downloaded and installed, we must configure how to resolve available\nversions. Resolving is configured through the ",(0,i.jsx)(n.code,{children:"[resolve]"})," section, which supports 2 patterns to\nresolve with: Git tags or a JSON manifest."]}),"\n",(0,i.jsx)(n.h4,{id:"git-tags",children:"Git tags"}),"\n",(0,i.jsx)(n.p,{children:"To resolve a list of available versions using Git tags, the following settings are available:"}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"git-url"})," (required) - The remote URL to fetch tags from."]}),"\n"]}),"\n",(0,i.jsx)(o.Z,{title:"protostar",data:{resolve:{gitUrl:"https://github.com/moonrepo/protostar"}}}),"\n",(0,i.jsx)(n.h4,{id:"json-manifest",children:"JSON manifest"}),"\n",(0,i.jsx)(n.p,{children:"To resolve a list of available versions using a JSON manifest, the following settings are available:"}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"manifest-url"})," (required) - A URL that returns a JSON response of all versions. This response\n",(0,i.jsx)(n.em,{children:"must be"})," an array of strings, or an array of objects."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"manifest-version-key"})," - If the response is an array of objects, this is the key to extract the\nversion from. If the response is an array of strings, this setting can be omitted. Defaults to\n",(0,i.jsx)(n.code,{children:"version"}),"."]}),"\n"]}),"\n",(0,i.jsx)(o.Z,{title:"protostar",data:{resolve:{manifestUrl:"https://someregistry.com/protostar/versions.json",manifestVersionKey:"latest_version"}}}),"\n",(0,i.jsxs)(n.h4,{id:"versions-and-aliases",children:["Versions and aliases",(0,i.jsx)(r.Z,{version:"0.36.0"})]}),"\n",(0,i.jsx)(n.p,{children:"As an alternative, we also support a static configuration of explicit versions and aliases. This is\nuseful if you have an internal tool that is relatively stable, or does not provide a means in which\nto extract version information."}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"versions"})," - A list of versions."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"aliases"})," - A mapping of alias names to versions."]}),"\n"]}),"\n",(0,i.jsx)(o.Z,{title:"protostar",data:{resolve:{versions:["1.2.3","1.2.4","1.2.5"],aliases:{stable:"1.2.4"}}}}),"\n",(0,i.jsx)(n.h4,{id:"version-patterns",children:"Version patterns"}),"\n",(0,i.jsxs)(n.p,{children:["When a version is found, either from a git tag or manifest key, we attempt to parse it into a\n",(0,i.jsx)(n.a,{href:"./version-spec",children:"valid version"})," using a Rust based regex pattern and the ",(0,i.jsx)(n.code,{children:"version-pattern"})," setting."]}),"\n",(0,i.jsxs)(n.p,{children:["This pattern uses named regex capture groups (",(0,i.jsx)(n.code,{children:"(?<name>...)"}),") to build the version, and to support\nfound versions that are not fully-qualified (they may be missing patch or minor versions). The\nfollowing groups are supported:"]}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"major"})," / ",(0,i.jsx)(n.code,{children:"year"})," - The major version number. Defaults to ",(0,i.jsx)(n.code,{children:"0"})," if missing."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"minor"})," / ",(0,i.jsx)(n.code,{children:"month"})," - The minor version number. Defaults to ",(0,i.jsx)(n.code,{children:"0"})," if missing."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"patch"})," / ",(0,i.jsx)(n.code,{children:"day"})," - The patch version number. Defaults to ",(0,i.jsx)(n.code,{children:"0"})," if missing."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"pre"}),' - The pre-release identifier, like "rc.0" or "alpha.0". Supports an optional leading ',(0,i.jsx)(n.code,{children:"-"}),".\nDoes nothing if missing."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"build"})," - The build metadata, like a timestamp. Supports an optional leading ",(0,i.jsx)(n.code,{children:"+"}),". Does nothing if\nmissing."]}),"\n"]}),"\n",(0,i.jsx)(o.Z,{title:"protostar",data:{resolve:{versionPattern:"^@protostar/cli@((?<major>\\d+)\\.(?<minor>\\d+)\\.(?<patch>\\d+))"}}}),"\n",(0,i.jsxs)(n.blockquote,{children:["\n",(0,i.jsxs)(n.p,{children:["If no named capture groups are found, the match at index ",(0,i.jsx)(n.code,{children:"1"})," is used as the version."]}),"\n"]}),"\n",(0,i.jsx)(n.h3,{id:"detecting-versions",children:"Detecting versions"}),"\n",(0,i.jsxs)(n.p,{children:["And lastly, we can configure how to ",(0,i.jsx)(n.a,{href:"./detection",children:"detect a version"})," contextually at runtime, using\nthe ",(0,i.jsx)(n.code,{children:"[detect]"})," setting. At this time, we only support 1 setting:"]}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"version-files"})," - A list of version files to extract from. The contents of these files can ",(0,i.jsx)(n.em,{children:"only"}),"\nbe the version string itself."]}),"\n"]}),"\n",(0,i.jsx)(o.Z,{title:"protostar",data:{detect:{versionFiles:[".protostar-version",".protostarrc"]}}})]})}function x(e={}){const{wrapper:n}={...(0,t.a)(),...e.components};return n?(0,i.jsx)(n,{...e,children:(0,i.jsx)(p,{...e})}):p(e)}},79022:(e,n,s)=>{s.d(n,{Z:()=>r});var i=s(9619),t=s(24246);function r(e){let{header:n,inline:s,updated:r,version:o}=e;return(0,t.jsx)(i.Z,{text:`v${o}`,variant:r?"success":"info",className:n?"absolute right-0 top-1.5":s?"inline-block":"ml-2"})}},59702:(e,n,s)=>{s.d(n,{Z:()=>p});var i=s(69373),t=s.n(i),r=s(97449),o=s(12126),l=s(52807),a=s(39798),c=s(33337),d=s(24246);function h(e,n){const s={};return Object.entries(e).forEach((e=>{let[i,r]=e;const o="arch"===n||"exes"===n?i:t()(i);s[o]=r&&"object"==typeof r&&!Array.isArray(r)?h(r,i):r})),s}function p(e){let{data:n={},title:s}=e;return(0,d.jsxs)(c.Z,{groupId:"non-wasm-type",defaultValue:"toml",values:[{label:"JSON",value:"json"},{label:"TOML",value:"toml"},{label:"YAML",value:"yaml"}],children:[(0,d.jsx)(a.Z,{value:"json",children:(0,d.jsx)(l.default,{language:"json",title:`${s}.json`,children:JSON.stringify(n,null,2)})}),(0,d.jsx)(a.Z,{value:"toml",children:(0,d.jsx)(l.default,{language:"toml",title:`${s}.toml`,children:r.ZP.stringify(h(n))})}),(0,d.jsx)(a.Z,{value:"yaml",children:(0,d.jsx)(l.default,{language:"yaml",title:`${s}.yaml`,children:o.ZP.stringify(n,{defaultKeyType:"PLAIN",defaultStringType:"QUOTE_SINGLE"})})})]})}},9619:(e,n,s)=>{s.d(n,{Z:()=>l});var i=s(40624),t=s(31792),r=s(24246);const o={failure:"bg-red-100 text-red-900",info:"bg-pink-100 text-pink-900",success:"bg-green-100 text-green-900",warning:"bg-orange-100 text-orange-900"};function l(e){let{className:n,icon:s,text:l,variant:a}=e;return(0,r.jsxs)("span",{className:(0,i.Z)("inline-flex items-center px-1 py-0.5 rounded text-xs font-bold uppercase",a?o[a]:"bg-gray-100 text-gray-800",n),children:[s&&(0,r.jsx)(t.Z,{icon:s,className:"mr-1"}),l]})}}}]);