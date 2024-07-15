"use strict";(self.webpackChunkwebsite=self.webpackChunkwebsite||[]).push([[37133],{32438:(e,n,s)=>{s.r(n),s.d(n,{assets:()=>c,contentTitle:()=>r,default:()=>l,frontMatter:()=>a,metadata:()=>i,toc:()=>h});var o=s(24246),t=s(71670);const a={title:"query hash",sidebar_label:"hash"},r=void 0,i={id:"commands/query/hash",title:"query hash",description:"Use the moon query hash sub-command to inspect the contents and sources of a generated hash, also",source:"@site/docs/commands/query/hash.mdx",sourceDirName:"commands/query",slug:"/commands/query/hash",permalink:"/docs/commands/query/hash",draft:!1,unlisted:!1,editUrl:"https://github.com/moonrepo/moon/tree/master/website/docs/commands/query/hash.mdx",tags:[],version:"current",frontMatter:{title:"query hash",sidebar_label:"hash"},sidebar:"docs",previous:{title:"query",permalink:"/docs/commands/query"},next:{title:"hash-diff",permalink:"/docs/commands/query/hash-diff"}},c={},h=[{value:"Options",id:"options",level:3},{value:"Configuration",id:"configuration",level:3}];function d(e){const n={a:"a",code:"code",h3:"h3",li:"li",p:"p",pre:"pre",ul:"ul",...(0,t.a)(),...e.components};return(0,o.jsxs)(o.Fragment,{children:[(0,o.jsxs)(n.p,{children:["Use the ",(0,o.jsx)(n.code,{children:"moon query hash"})," sub-command to inspect the contents and sources of a generated hash, also\nknown as the hash manifest. This is extremely useful in debugging task inputs."]}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{className:"language-shell",children:"$ moon query hash 0b55b234f1018581c45b00241d7340dc648c63e639fbafdaf85a4cd7e718fdde\n\n# Query hash using short form\n$ moon query hash 0b55b234\n"})}),"\n",(0,o.jsx)(n.p,{children:"By default, this will output the contents of the hash manifest (which is JSON), and the fully\nqualified resolved hash."}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{className:"language-json",children:'Hash: 0b55b234f1018581c45b00241d7340dc648c63e639fbafdaf85a4cd7e718fdde\n\n{\n  "command": "build",\n  "args": ["./build"]\n  // ...\n}\n'})}),"\n",(0,o.jsxs)(n.p,{children:["The command can also be output raw JSON by passing the ",(0,o.jsx)(n.code,{children:"--json"})," flag."]}),"\n",(0,o.jsx)(n.h3,{id:"options",children:"Options"}),"\n",(0,o.jsxs)(n.ul,{children:["\n",(0,o.jsxs)(n.li,{children:[(0,o.jsx)(n.code,{children:"--json"})," - Display the diff in JSON format."]}),"\n"]}),"\n",(0,o.jsx)(n.h3,{id:"configuration",children:"Configuration"}),"\n",(0,o.jsxs)(n.ul,{children:["\n",(0,o.jsxs)(n.li,{children:[(0,o.jsx)(n.a,{href:"../../config/workspace#hasher",children:(0,o.jsx)(n.code,{children:"hasher"})})," in ",(0,o.jsx)(n.code,{children:".moon/workspace.yml"})]}),"\n"]})]})}function l(e={}){const{wrapper:n}={...(0,t.a)(),...e.components};return n?(0,o.jsx)(n,{...e,children:(0,o.jsx)(d,{...e})}):d(e)}},71670:(e,n,s)=>{s.d(n,{Z:()=>i,a:()=>r});var o=s(27378);const t={},a=o.createContext(t);function r(e){const n=o.useContext(a);return o.useMemo((function(){return"function"==typeof e?e(n):{...n,...e}}),[n,e])}function i(e){let n;return n=e.disableParentContext?"function"==typeof e.components?e.components(t):e.components||t:r(e.components),o.createElement(a.Provider,{value:n},e.children)}}}]);