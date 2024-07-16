"use strict";(self.webpackChunkwebsite=self.webpackChunkwebsite||[]).push([[83508],{38294:(e,n,o)=>{o.r(n),o.d(n,{assets:()=>d,contentTitle:()=>r,default:()=>p,frontMatter:()=>l,metadata:()=>c,toc:()=>i});var t=o(24246),s=o(71670);const l={title:"clean"},r=void 0,c={id:"proto/commands/clean",title:"clean",description:"The proto clean command can be used to uninstall stale and unused tools or plugins. By default, it",source:"@site/docs/proto/commands/clean.mdx",sourceDirName:"proto/commands",slug:"/proto/commands/clean",permalink:"/docs/proto/commands/clean",draft:!1,unlisted:!1,editUrl:"https://github.com/moonrepo/moon/tree/master/website/docs/proto/commands/clean.mdx",tags:[],version:"current",frontMatter:{title:"clean"},sidebar:"proto",previous:{title:"bin",permalink:"/docs/proto/commands/bin"},next:{title:"completions",permalink:"/docs/proto/commands/completions"}},d={},i=[{value:"Options",id:"options",level:3}];function a(e){const n={code:"code",h3:"h3",li:"li",p:"p",pre:"pre",ul:"ul",...(0,s.a)(),...e.components};return(0,t.jsxs)(t.Fragment,{children:[(0,t.jsxs)(n.p,{children:["The ",(0,t.jsx)(n.code,{children:"proto clean"})," command can be used to uninstall stale and unused tools or plugins. By default, it\nwill remove items that haven't been used in the last 30 days."]}),"\n",(0,t.jsx)(n.pre,{children:(0,t.jsx)(n.code,{className:"language-shell",children:"$ proto clean\n"})}),"\n",(0,t.jsx)(n.p,{children:"Furthermore, the command can be used to purge a tool, which will remove it entirely from proto, or\npurge all downloaded plugins."}),"\n",(0,t.jsx)(n.pre,{children:(0,t.jsx)(n.code,{className:"language-shell",children:"# Delete node from proto\n$ proto clean --purge node\n\n# Delete all plugins\n$ proto clean --purge-plugins\n"})}),"\n",(0,t.jsx)(n.h3,{id:"options",children:"Options"}),"\n",(0,t.jsxs)(n.ul,{children:["\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:"--days"})," - Number of days before a tool is considered stale."]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:"--purge"})," - Purge and delete the installed tool by ID (",(0,t.jsx)(n.code,{children:"~/.proto/tools/<id>"}),")."]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:"--purge-plugins"})," - Purge and delete all downloaded plugins (",(0,t.jsx)(n.code,{children:"~/.proto/plugins"}),")."]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:"--yes"})," - Avoid and confirm all prompts."]}),"\n"]})]})}function p(e={}){const{wrapper:n}={...(0,s.a)(),...e.components};return n?(0,t.jsx)(n,{...e,children:(0,t.jsx)(a,{...e})}):a(e)}},71670:(e,n,o)=>{o.d(n,{Z:()=>c,a:()=>r});var t=o(27378);const s={},l=t.createContext(s);function r(e){const n=t.useContext(l);return t.useMemo((function(){return"function"==typeof e?e(n):{...n,...e}}),[n,e])}function c(e){let n;return n=e.disableParentContext?"function"==typeof e.components?e.components(s):e.components||s:r(e.components),t.createElement(l.Provider,{value:n},e.children)}}}]);