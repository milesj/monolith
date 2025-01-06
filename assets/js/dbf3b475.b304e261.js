"use strict";(self.webpackChunkwebsite=self.webpackChunkwebsite||[]).push([[70067],{28547:(e,n,t)=>{t.r(n),t.d(n,{assets:()=>l,contentTitle:()=>a,default:()=>h,frontMatter:()=>i,metadata:()=>r,toc:()=>c});var s=t(24246),o=t(71670);const i={slug:"moon-v1.31",title:"moon v1.31 - Toolchain progress, glob-based targets, task & remote cache improvements",authors:["milesj"],tags:["platform","toolchain","glob","task","target","remote","cache","ci"],image:"./img/moon/v1.31.png"},a=void 0,r={permalink:"/blog/moon-v1.31",editUrl:"https://github.com/moonrepo/moon/tree/master/website/blog/2025-01-06_moon-v1.31.mdx",source:"@site/blog/2025-01-06_moon-v1.31.mdx",title:"moon v1.31 - Toolchain progress, glob-based targets, task & remote cache improvements",description:"Happy new years everyone \ud83c\udf89! In this release, we've landed a handful of quality-of-life",date:"2025-01-06T00:00:00.000Z",tags:[{inline:!0,label:"platform",permalink:"/blog/tags/platform"},{inline:!0,label:"toolchain",permalink:"/blog/tags/toolchain"},{inline:!0,label:"glob",permalink:"/blog/tags/glob"},{inline:!0,label:"task",permalink:"/blog/tags/task"},{inline:!0,label:"target",permalink:"/blog/tags/target"},{inline:!0,label:"remote",permalink:"/blog/tags/remote"},{inline:!0,label:"cache",permalink:"/blog/tags/cache"},{inline:!0,label:"ci",permalink:"/blog/tags/ci"}],readingTime:5.555,hasTruncateMarker:!0,authors:[{name:"Miles Johnson",title:"Founder, developer",url:"https://github.com/milesj",imageURL:"/img/authors/miles.jpg",key:"milesj"}],frontMatter:{slug:"moon-v1.31",title:"moon v1.31 - Toolchain progress, glob-based targets, task & remote cache improvements",authors:["milesj"],tags:["platform","toolchain","glob","task","target","remote","cache","ci"],image:"./img/moon/v1.31.png"},unlisted:!1,nextItem:{title:"proto v0.44 - New terminal user interface and versions command",permalink:"/blog/proto-v0.44"}},l={image:t(84634).Z,authorsImageUrls:[void 0]},c=[{value:"Goodbye platform, hello toolchain",id:"goodbye-platform-hello-toolchain",level:2},{value:"Run tasks using glob-based targets",id:"run-tasks-using-glob-based-targets",level:2},{value:"Task improvements",id:"task-improvements",level:2},{value:"Inferring inputs",id:"inferring-inputs",level:3},{value:"Always run in CI",id:"always-run-in-ci",level:3},{value:"Remote cache improvements",id:"remote-cache-improvements",level:2},{value:"Zstandard compression",id:"zstandard-compression",level:3},{value:"Symlinking on Windows",id:"symlinking-on-windows",level:3},{value:"Sunsetting moonbase",id:"sunsetting-moonbase",level:3},{value:"Other changes",id:"other-changes",level:2}];function d(e){const n={a:"a",admonition:"admonition",code:"code",em:"em",h2:"h2",h3:"h3",li:"li",p:"p",pre:"pre",ul:"ul",...(0,o.a)(),...e.components};return(0,s.jsxs)(s.Fragment,{children:[(0,s.jsx)(n.p,{children:"Happy new years everyone \ud83c\udf89! In this release, we've landed a handful of quality-of-life\nimprovements."}),"\n",(0,s.jsx)(n.h2,{id:"goodbye-platform-hello-toolchain",children:"Goodbye platform, hello toolchain"}),"\n",(0,s.jsxs)(n.p,{children:["In preparation for toolchain WASM plugins, we've had to rewrite and rethink a lot of the internals\nof moon. Once such feature is the concept of a \"platform\", which is primarily interacted with\nthrough a task's ",(0,s.jsx)(n.a,{href:"/docs/config/project#platform-1",children:(0,s.jsx)(n.code,{children:"platform"})})," setting or a project's\n",(0,s.jsx)(n.a,{href:"/docs/config/project#platform",children:(0,s.jsx)(n.code,{children:"platform"})})," setting."]}),"\n",(0,s.jsxs)(n.p,{children:['We do our best to detect the language and runtime (the "platform") that a project or task belongs\nto. This is important as it determines what tools to install, paths to include in ',(0,s.jsx)(n.code,{children:"PATH"}),", and much\nmore. However, there are situations where our detection fails, or you need to be explicit, so the\n",(0,s.jsx)(n.code,{children:"platform"})," settings exist."]}),"\n",(0,s.jsx)(n.p,{children:"The new toolchain system is much more powerful, but it works quite differently, so we're slowly\nmaking changes within each release before flipping the switch. In this release, we are deprecating\nthe concept of the platform, and renaming everything to toolchain. The following changes were made:"}),"\n",(0,s.jsxs)(n.ul,{children:["\n",(0,s.jsxs)(n.li,{children:["Deprecated the top-level ",(0,s.jsx)(n.code,{children:"platform"})," setting in ",(0,s.jsx)(n.code,{children:"moon.yml"}),", use ",(0,s.jsx)(n.code,{children:"toolchain.default"})," instead.","\n",(0,s.jsxs)(n.ul,{children:["\n",(0,s.jsxs)(n.li,{children:["Additionally, the toolchain can now be inferred from the top-level ",(0,s.jsx)(n.code,{children:"language"})," setting and any\nconfig files in the project/workspace root. This pattern is preferred when possible."]}),"\n"]}),"\n"]}),"\n",(0,s.jsxs)(n.li,{children:["Deprecated the ",(0,s.jsx)(n.code,{children:"platform"})," task setting, use ",(0,s.jsx)(n.code,{children:"toolchain"})," instead."]}),"\n",(0,s.jsxs)(n.li,{children:["Deprecated the ",(0,s.jsx)(n.code,{children:"taskPlatform"})," query field, use ",(0,s.jsx)(n.code,{children:"taskToolchain"})," instead."]}),"\n",(0,s.jsxs)(n.li,{children:["Deprecated the ",(0,s.jsx)(n.code,{children:"--platform"})," option for ",(0,s.jsx)(n.code,{children:"moon query tasks"}),", use ",(0,s.jsx)(n.code,{children:"--toolchain"})," instead."]}),"\n",(0,s.jsxs)(n.li,{children:["Deprecated the ",(0,s.jsx)(n.code,{children:"$taskPlatform"})," token, use ",(0,s.jsx)(n.code,{children:"$taskToolchain"})," instead."]}),"\n"]}),"\n",(0,s.jsx)(n.admonition,{type:"warning",children:(0,s.jsx)(n.p,{children:"On the surface these two features look the same, but internally they are quite different. We've done\nour best to support backwards compatibility, but there may be some edge cases that our testing suite\ndid not cover. If you run into any problems, mainly tasks being associated with the wrong toolchain,\nplease report an issue!"})}),"\n",(0,s.jsx)(n.h2,{id:"run-tasks-using-glob-based-targets",children:"Run tasks using glob-based targets"}),"\n",(0,s.jsxs)(n.p,{children:["This has been a request from the community for sometime, as it fills a gap that running multiple\ntasks with a non-project scope, or running tasks with a query, simply couldn't achieve. For example,\nsay you had ",(0,s.jsx)(n.code,{children:"build-debug"})," and ",(0,s.jsx)(n.code,{children:"build-release"})," tasks, and wanted to future-proof it for potential new\nbuild related tasks."]}),"\n",(0,s.jsxs)(n.p,{children:["Before this release, you would need to explicitly list all targets in\n",(0,s.jsx)(n.a,{href:"/docs/commands/run",children:(0,s.jsx)(n.code,{children:"moon run"})})," or ",(0,s.jsx)(n.a,{href:"/docs/commands/ci",children:(0,s.jsx)(n.code,{children:"moon ci"})}),", but with globs, you can achieve\nthe same affect with 1 glob target."]}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-shell",children:"# Before\n$ moon run :build-debug :build-release\n\n# After\n$ moon run ':build-*'\n"})}),"\n",(0,s.jsx)(n.p,{children:"Furthermore, glob syntax can also be applied to the project scope, allowing you to filter the target\ninstead of applying to all projects."}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-shell",children:"$ moon run '*-{app,lib}:build-*'\n"})}),"\n",(0,s.jsxs)(n.p,{children:["Because these glob targets are real globs, they support all the\n",(0,s.jsx)(n.a,{href:"/docs/concepts/file-pattern#globs",children:"same syntax"})," as other glob related functionality, but we suggest\nkeeping it simple and sticking to ",(0,s.jsx)(n.code,{children:"*"}),", ",(0,s.jsx)(n.code,{children:"[]"}),", and ",(0,s.jsx)(n.code,{children:"{}"}),"."]}),"\n",(0,s.jsx)(n.admonition,{type:"info",children:(0,s.jsx)(n.p,{children:"Be sure to quote targets that contain glob syntax, otherwise your shell native glob expansion may\ntrigger instead, or your shell may fail with an error."})}),"\n",(0,s.jsx)(n.h2,{id:"task-improvements",children:"Task improvements"}),"\n",(0,s.jsx)(n.p,{children:"We also spent some time improving the ergonomics of tasks, our most important feature."}),"\n",(0,s.jsx)(n.h3,{id:"inferring-inputs",children:"Inferring inputs"}),"\n",(0,s.jsxs)(n.p,{children:["Up until now, you had to explicitly configure the ",(0,s.jsx)(n.a,{href:"/docs/config/project#inputs",children:(0,s.jsx)(n.code,{children:"inputs"})}),' of a task.\nThis can be very tedious, so we\'re looking into ways to automate this. The first is through a new\nfeature we are calling "inferring inputs from task parameters", where we automatically include\ninputs from any file group token functions and substituted environment variables, found within\n',(0,s.jsx)(n.code,{children:"command"}),", ",(0,s.jsx)(n.code,{children:"script"}),", ",(0,s.jsx)(n.code,{children:"args"}),", or ",(0,s.jsx)(n.code,{children:"env"}),"."]}),"\n",(0,s.jsx)(n.p,{children:"To demonstate this, here's a task that utilizes file group tokens in previous releases."}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-yaml",metastring:'title="moon.yml"',children:"tasks:\n  lint:\n    command: 'lint @group(sources) @group(tests)'\n    inputs:\n      - '@group(sources)'\n      - '@group(tests)'\n"})}),"\n",(0,s.jsxs)(n.p,{children:["As you can immediately tell, there's a fair bit of duplication here. Going forward, the tokens found\nwithin ",(0,s.jsx)(n.code,{children:"inputs"})," can be omitted, as we can infer that the files defined in the ",(0,s.jsx)(n.code,{children:"sources"})," and ",(0,s.jsx)(n.code,{children:"tests"}),"\nfile groups should be inputs. The task above can simply be rewritten as."]}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-yaml",metastring:'title="moon.yml"',children:"tasks:\n  lint:\n    command: 'lint @group(sources) @group(tests)'\n"})}),"\n",(0,s.jsxs)(n.p,{children:["Useful right? However, if you do ",(0,s.jsx)(n.em,{children:"not"})," want this functionality, you can disable it with the new task\noption ",(0,s.jsx)(n.a,{href:"/docs/config/project#inferinputs",children:(0,s.jsx)(n.code,{children:"inferInputs"})})," (which is enabled by default)."]}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-yaml",metastring:'title="moon.yml"',children:"tasks:\n  lint:\n    # ...\n    options:\n      inferInputs: false\n"})}),"\n",(0,s.jsx)(n.h3,{id:"always-run-in-ci",children:"Always run in CI"}),"\n",(0,s.jsxs)(n.p,{children:["The ",(0,s.jsx)(n.a,{href:"/docs/config/project#runinci",children:(0,s.jsx)(n.code,{children:"runInCI"})})," task option pairs nicely with the ",(0,s.jsx)(n.code,{children:"moon ci"})," command,\nas it does most of the heavy lifting in determining what tasks to run based on affected/touched\nfiles. However, there are sometimes situations where a task should ",(0,s.jsx)(n.em,{children:"always"})," run in CI, regardless of\nwhether it was affected or not."]}),"\n",(0,s.jsxs)(n.p,{children:["This isn't currently possible in moon, until now! We've updated the ",(0,s.jsx)(n.code,{children:"runInCI"}),' option to support a\nnew value, "always", which will always run the task in CI!']}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-yaml",metastring:'title="moon.yml"',children:"tasks:\n  build:\n    # ...\n    options:\n      runInCI: 'always'\n"})}),"\n",(0,s.jsx)(n.h2,{id:"remote-cache-improvements",children:"Remote cache improvements"}),"\n",(0,s.jsxs)(n.p,{children:["In our last release, v1.30, we released\n",(0,s.jsx)(n.a,{href:"./moon-v1.30#unstable-self-hosted-remote-caching",children:"unstable support for self-hosted remote caching"}),".\nWhile still unstable in this release, we've landed more improvements."]}),"\n",(0,s.jsx)(n.h3,{id:"zstandard-compression",children:"Zstandard compression"}),"\n",(0,s.jsxs)(n.p,{children:["We've added a new setting,\n",(0,s.jsx)(n.a,{href:"/docs/config/workspace#compression",children:(0,s.jsx)(n.code,{children:"unstable_remote.cache.compression"})}),", that defines a\ncompression format to use when uploading and downloading blobs. At this time, we only support ",(0,s.jsx)(n.code,{children:"zstd"}),"\nas an option, which is ",(0,s.jsx)(n.em,{children:"not"})," enabled by default."]}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-yaml",metastring:'title=".moon/workspace.yml"',children:"unstable_remote:\n  cache:\n    compression: 'zstd'\n"})}),"\n",(0,s.jsxs)(n.p,{children:["If you're using ",(0,s.jsx)(n.code,{children:"bazel-remote"})," as your cache server, you'll also need to run it with zstandard\nenabled."]}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-shell",children:"$ bazel-remote --dir /path/to/moon-cache --max_size 10 --storage_mode zstd --grpc_address 0.0.0.0:9092\n"})}),"\n",(0,s.jsx)(n.h3,{id:"symlinking-on-windows",children:"Symlinking on Windows"}),"\n",(0,s.jsx)(n.p,{children:"In the previous release, if we encountered an output blob that should be created as a symlink, we\nwould simply copy the file contents on Windows when restoring instead of symlinking. On Unix, these\noutputs were symlinked correctly."}),"\n",(0,s.jsxs)(n.p,{children:["The reason for this, is that symlinks require\n",(0,s.jsx)(n.a,{href:"https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/security-policy-settings/create-symbolic-links",children:"privileged access on Windows"}),"\nto function correctly. We felt that abiding the REAPI specification was more important than the\nprivileged access requirement, so if you're on Windows, be sure to allow/enable symlinks on each\nmachine."]}),"\n",(0,s.jsx)(n.h3,{id:"sunsetting-moonbase",children:"Sunsetting moonbase"}),"\n",(0,s.jsxs)(n.p,{children:["Since we're migrating to and advocating for the self-hosted remote caching solution, we will be\nsunsetting our ",(0,s.jsx)(n.a,{href:"/moonbase",children:"moonbase"})," product hosted at ",(0,s.jsx)(n.a,{href:"https://moonrepo.app",children:"https://moonrepo.app"})," on March 31st. All\nactive subscriptions will be cancelled at the end of February, but caching will continue to work,\nalbeit at the unpaid plan limits. We suggest migrating to the self-hosted solution before then!"]}),"\n",(0,s.jsx)(n.h2,{id:"other-changes",children:"Other changes"}),"\n",(0,s.jsxs)(n.p,{children:["View the ",(0,s.jsx)(n.a,{href:"https://github.com/moonrepo/moon/releases/tag/v1.31.0",children:"official release"})," for a full list\nof changes."]}),"\n",(0,s.jsxs)(n.ul,{children:["\n",(0,s.jsxs)(n.li,{children:["Added glob support (and ",(0,s.jsx)(n.code,{children:"glob://"}),") to ",(0,s.jsx)(n.code,{children:"generator.templates"})," in ",(0,s.jsx)(n.code,{children:".moon/workspace.yml"}),", allowing you\nto glob for your codegen template locations."]}),"\n",(0,s.jsxs)(n.li,{children:["Added a ",(0,s.jsx)(n.code,{children:"--filter"})," option to ",(0,s.jsx)(n.code,{children:"moon templates"}),"."]}),"\n",(0,s.jsxs)(n.li,{children:["Updated the ",(0,s.jsx)(n.code,{children:"extends"})," setting in ",(0,s.jsx)(n.code,{children:".moon/workspace.yml"}),", ",(0,s.jsx)(n.code,{children:"toolchain.yml"}),", and ",(0,s.jsx)(n.code,{children:"tasks.yml"}),", to\nsupport a list of files/URLs to extend."]}),"\n",(0,s.jsx)(n.li,{children:"Updated toolchain dependency installs to retry up to 3 attempts if the install command fails."}),"\n",(0,s.jsx)(n.li,{children:"Improved the task output prefixing logic."}),"\n"]})]})}function h(e={}){const{wrapper:n}={...(0,o.a)(),...e.components};return n?(0,s.jsx)(n,{...e,children:(0,s.jsx)(d,{...e})}):d(e)}},84634:(e,n,t)=>{t.d(n,{Z:()=>s});const s=t.p+"assets/images/v1.31-697f72b088308b76b09f07a676a82217.png"},71670:(e,n,t)=>{t.d(n,{Z:()=>r,a:()=>a});var s=t(27378);const o={},i=s.createContext(o);function a(e){const n=s.useContext(i);return s.useMemo((function(){return"function"==typeof e?e(n):{...n,...e}}),[n,e])}function r(e){let n;return n=e.disableParentContext?"function"==typeof e.components?e.components(o):e.components||o:a(e.components),s.createElement(i.Provider,{value:n},e.children)}}}]);