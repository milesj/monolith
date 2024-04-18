"use strict";(self.webpackChunkwebsite=self.webpackChunkwebsite||[]).push([[55126],{34558:(e,t,r)=>{r.r(t),r.d(t,{assets:()=>l,contentTitle:()=>s,default:()=>u,frontMatter:()=>i,metadata:()=>a,toc:()=>m});var o=r(24246),n=r(71670);const i={slug:"proto-v0.28",title:"proto v0.28 - Upgraded WASM runtime",authors:["milesj"],tags:["proto","wasm","runtime","extism"]},s=void 0,a={permalink:"/blog/proto-v0.28",editUrl:"https://github.com/moonrepo/moon/tree/master/website/blog/2024-01-17_proto-v0.28.mdx",source:"@site/blog/2024-01-17_proto-v0.28.mdx",title:"proto v0.28 - Upgraded WASM runtime",description:"This is a small release that primarily upgrades our WASM runtime.",date:"2024-01-17T00:00:00.000Z",formattedDate:"January 17, 2024",tags:[{label:"proto",permalink:"/blog/tags/proto"},{label:"wasm",permalink:"/blog/tags/wasm"},{label:"runtime",permalink:"/blog/tags/runtime"},{label:"extism",permalink:"/blog/tags/extism"}],readingTime:.98,hasTruncateMarker:!0,authors:[{name:"Miles Johnson",title:"Founder, developer",url:"https://github.com/milesj",imageURL:"/img/authors/miles.jpg",key:"milesj"}],frontMatter:{slug:"proto-v0.28",title:"proto v0.28 - Upgraded WASM runtime",authors:["milesj"],tags:["proto","wasm","runtime","extism"]},unlisted:!1,prevItem:{title:"proto v0.29 - Better environment support",permalink:"/blog/proto-v0.29"},nextItem:{title:"What's in store for 2024",permalink:"/blog/2024-roadmap"}},l={authorsImageUrls:[void 0]},m=[{value:"Upgraded WASM runtime",id:"upgraded-wasm-runtime",level:2},{value:"Other changes",id:"other-changes",level:2}];function p(e){const t={a:"a",em:"em",h2:"h2",li:"li",p:"p",ul:"ul",...(0,n.a)(),...e.components};return(0,o.jsxs)(o.Fragment,{children:[(0,o.jsx)(t.p,{children:"This is a small release that primarily upgrades our WASM runtime."}),"\n",(0,o.jsx)(t.h2,{id:"upgraded-wasm-runtime",children:"Upgraded WASM runtime"}),"\n",(0,o.jsxs)(t.p,{children:["proto utilizes ",(0,o.jsx)(t.a,{href:"https://extism.org/",children:"Extism"})," for our WASM plugin architecture, which internally uses\n",(0,o.jsx)(t.a,{href:"https://wasmtime.dev/",children:"wasmtime"})," as our execution runtime. Up until this point, we've been using a\nbeta release of Extism, v0.5, which has worked quite nicely. We've also been working closely with\nthe Extism team to report bugs, provide feedback, and help improve the project. Once such feature\nwas the massive performance gains in ",(0,o.jsx)(t.a,{href:"./proto-v0.24",children:"proto v0.24"}),"."]}),"\n",(0,o.jsxs)(t.p,{children:["Thanks to all the hard work from the Extism team over the past year, an official v1.0 was released.\nBecause this was a major release, it did include breaking changes around the WASM runtime, and as\nsuch, proto WASM plugins before v0.28 are ",(0,o.jsx)(t.em,{children:"no longer compatible"}),", and will need to be recompiled\nwith the latest PDKs. Our proto TOML plugins are not affected."]}),"\n",(0,o.jsx)(t.h2,{id:"other-changes",children:"Other changes"}),"\n",(0,o.jsxs)(t.p,{children:["View the ",(0,o.jsx)(t.a,{href:"https://github.com/moonrepo/proto/releases/tag/v0.28.0",children:"official release"})," for a full list\nof changes."]}),"\n",(0,o.jsxs)(t.ul,{children:["\n",(0,o.jsx)(t.li,{children:"Will now display an upgrade message when the current proto version is out of date."}),"\n",(0,o.jsx)(t.li,{children:"Improved error messages to include plugin specific information."}),"\n",(0,o.jsx)(t.li,{children:'Updated our "last used at" logic to avoid race conditions with the tool manifest.'}),"\n"]})]})}function u(e={}){const{wrapper:t}={...(0,n.a)(),...e.components};return t?(0,o.jsx)(t,{...e,children:(0,o.jsx)(p,{...e})}):p(e)}},71670:(e,t,r)=>{r.d(t,{Z:()=>a,a:()=>s});var o=r(27378);const n={},i=o.createContext(n);function s(e){const t=o.useContext(i);return o.useMemo((function(){return"function"==typeof e?e(t):{...t,...e}}),[t,e])}function a(e){let t;return t=e.disableParentContext?"function"==typeof e.components?e.components(n):e.components||n:s(e.components),o.createElement(i.Provider,{value:t},e.children)}}}]);