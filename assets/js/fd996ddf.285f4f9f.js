"use strict";(self.webpackChunkwebsite=self.webpackChunkwebsite||[]).push([[5190],{42849:(e,o,n)=>{n.r(o),n.d(o,{assets:()=>a,contentTitle:()=>i,default:()=>d,frontMatter:()=>s,metadata:()=>r,toc:()=>h});var t=n(24246),l=n(71670);const s={slug:"proto-v0.38",title:"proto v0.38 - Shell activation / hooks",authors:["milesj"],tags:["shell","activate","hook","murex","env","path"]},i=void 0,r={permalink:"/blog/proto-v0.38",editUrl:"https://github.com/moonrepo/moon/tree/master/website/blog/2024-07-07_proto-v0.38.mdx",source:"@site/blog/2024-07-07_proto-v0.38.mdx",title:"proto v0.38 - Shell activation / hooks",description:"In this release, we're introducing a long requested feature, shell hooks!",date:"2024-07-07T00:00:00.000Z",tags:[{inline:!0,label:"shell",permalink:"/blog/tags/shell"},{inline:!0,label:"activate",permalink:"/blog/tags/activate"},{inline:!0,label:"hook",permalink:"/blog/tags/hook"},{inline:!0,label:"murex",permalink:"/blog/tags/murex"},{inline:!0,label:"env",permalink:"/blog/tags/env"},{inline:!0,label:"path",permalink:"/blog/tags/path"}],readingTime:1.75,hasTruncateMarker:!0,authors:[{name:"Miles Johnson",title:"Founder, developer",url:"https://github.com/milesj",imageURL:"/img/authors/miles.jpg",key:"milesj"}],frontMatter:{slug:"proto-v0.38",title:"proto v0.38 - Shell activation / hooks",authors:["milesj"],tags:["shell","activate","hook","murex","env","path"]},unlisted:!1,prevItem:{title:"moon v1.27 - Task scripts, Docker settings, and more",permalink:"/blog/moon-v1.27"},nextItem:{title:"moon v1.26 - New experimental pipeline, trace profiles, and more",permalink:"/blog/moon-v1.26"}},a={authorsImageUrls:[void 0]},h=[{value:"New experimental shell activation workflow",id:"new-experimental-shell-activation-workflow",level:2},{value:"How it works",id:"how-it-works",level:3},{value:"Unlocked features",id:"unlocked-features",level:3},{value:"Other changes",id:"other-changes",level:2}];function c(e){const o={a:"a",code:"code",h2:"h2",h3:"h3",li:"li",p:"p",pre:"pre",ul:"ul",...(0,l.a)(),...e.components};return(0,t.jsxs)(t.Fragment,{children:[(0,t.jsx)(o.p,{children:"In this release, we're introducing a long requested feature, shell hooks!"}),"\n",(0,t.jsx)(o.h2,{id:"new-experimental-shell-activation-workflow",children:"New experimental shell activation workflow"}),"\n",(0,t.jsxs)(o.p,{children:["You've most likely used another version manager before proto, and may have used a workflow where\n",(0,t.jsx)(o.code,{children:"PATH"})," was automatically updated with versioned binaries of specific tools, or environment variables\nwere injected into your shell. This functionality is what's known as shell hooks, and proto now has\nexperimental support for them through a feature known as\n",(0,t.jsx)(o.a,{href:"/docs/proto/workflows#shell-activation",children:"shell activation"}),"!"]}),"\n",(0,t.jsx)(o.h3,{id:"how-it-works",children:"How it works"}),"\n",(0,t.jsxs)(o.p,{children:["For example, say you're using Zsh as your shell. You could now append the following expression at\nthe bottom of your shell profile, which evaluates our new\n",(0,t.jsx)(o.a,{href:"/docs/proto/commands/activate",children:(0,t.jsx)(o.code,{children:"proto activate"})})," command."]}),"\n",(0,t.jsx)(o.pre,{children:(0,t.jsx)(o.code,{className:"language-shell",children:'eval "$(proto activate zsh)"\n'})}),"\n",(0,t.jsxs)(o.p,{children:["When the current working directory changes (via ",(0,t.jsx)(o.code,{children:"cd"}),"), or the CLI prompt changes, this activation\nworkflow will trigger the following:"]}),"\n",(0,t.jsxs)(o.ul,{children:["\n",(0,t.jsx)(o.li,{children:"Download and install necessary proto plugins (if they do not exist)"}),"\n",(0,t.jsxs)(o.li,{children:["Load and resolve all ",(0,t.jsx)(o.code,{children:".prototools"})," configurations up the file system"]}),"\n",(0,t.jsx)(o.li,{children:"Detect and resolve versions for all configured tools"}),"\n",(0,t.jsxs)(o.li,{children:["Export environment variables defined in ",(0,t.jsx)(o.code,{children:"[env]"})," and ",(0,t.jsx)(o.code,{children:"[tools.*.env]"})]}),"\n",(0,t.jsxs)(o.li,{children:["Prepend ",(0,t.jsx)(o.code,{children:"PATH"})," with binary directories for detected tools"]}),"\n"]}),"\n",(0,t.jsx)(o.p,{children:"Pretty awesome right? We think so. But as mentioned above, this feature is highly experimental, and\nmay not work properly across all shells (we're unable to test everything). So if you run into an\nissue, please report it!"}),"\n",(0,t.jsx)(o.h3,{id:"unlocked-features",children:"Unlocked features"}),"\n",(0,t.jsx)(o.p,{children:"This new workflow unlocks some functionality that was previously not possible with proto shims/bins\ndirectly, and they are:"}),"\n",(0,t.jsxs)(o.ul,{children:["\n",(0,t.jsxs)(o.li,{children:["Directory paths to globally installed packages are now automatically prepended to ",(0,t.jsx)(o.code,{children:"PATH"}),".\nPreviously, you would need to add them manually. This was non-trivial if they were installed to\nversioned locations."]}),"\n",(0,t.jsxs)(o.li,{children:["Directory paths to pre-installed binaries within a tool are also prepended to ",(0,t.jsx)(o.code,{children:"PATH"}),". For example,\nRust/Cargo and Python provide a lot of executables that were ignored by our shims."]}),"\n",(0,t.jsxs)(o.li,{children:["This workflow is 1 step closer to replacing ",(0,t.jsx)(o.a,{href:"https://direnv.net/",children:"direnv"})," itself."]}),"\n"]}),"\n",(0,t.jsx)(o.h2,{id:"other-changes",children:"Other changes"}),"\n",(0,t.jsxs)(o.p,{children:["View the ",(0,t.jsx)(o.a,{href:"https://github.com/moonrepo/proto/releases/tag/v0.38.0",children:"official release"})," for a full list\nof changes."]}),"\n",(0,t.jsxs)(o.ul,{children:["\n",(0,t.jsxs)(o.li,{children:["Added support for ",(0,t.jsx)(o.a,{href:"https://murex.rocks/",children:"murex"})," shells."]}),"\n",(0,t.jsxs)(o.li,{children:["Added a ",(0,t.jsx)(o.code,{children:"--include-global"})," flag to ",(0,t.jsx)(o.code,{children:"proto use"}),", that will also install globally configured tools."]}),"\n"]})]})}function d(e={}){const{wrapper:o}={...(0,l.a)(),...e.components};return o?(0,t.jsx)(o,{...e,children:(0,t.jsx)(c,{...e})}):c(e)}},71670:(e,o,n)=>{n.d(o,{Z:()=>r,a:()=>i});var t=n(27378);const l={},s=t.createContext(l);function i(e){const o=t.useContext(s);return t.useMemo((function(){return"function"==typeof e?e(o):{...o,...e}}),[o,e])}function r(e){let o;return o=e.disableParentContext?"function"==typeof e.components?e.components(l):e.components||l:i(e.components),t.createElement(s.Provider,{value:o},e.children)}}}]);