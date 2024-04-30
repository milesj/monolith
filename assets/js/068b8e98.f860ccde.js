"use strict";(self.webpackChunkwebsite=self.webpackChunkwebsite||[]).push([[35666],{95807:(e,n,i)=>{i.r(n),i.d(n,{assets:()=>a,contentTitle:()=>c,default:()=>h,frontMatter:()=>s,metadata:()=>r,toc:()=>l});var o=i(24246),t=i(71670);const s={title:"ci"},c=void 0,r={id:"commands/ci",title:"ci",description:"The moon ci command is a special command that should be ran in a continuous integration (CI)",source:"@site/docs/commands/ci.mdx",sourceDirName:"commands",slug:"/commands/ci",permalink:"/docs/commands/ci",draft:!1,unlisted:!1,editUrl:"https://github.com/moonrepo/moon/tree/master/website/docs/commands/ci.mdx",tags:[],version:"current",frontMatter:{title:"ci"},sidebar:"docs",previous:{title:"bin",permalink:"/docs/commands/bin"},next:{title:"check",permalink:"/docs/commands/check"}},a={},l=[{value:"Arguments",id:"arguments",level:3},{value:"Options",id:"options",level:3}];function d(e){const n={a:"a",blockquote:"blockquote",code:"code",h3:"h3",li:"li",p:"p",pre:"pre",ul:"ul",...(0,t.a)(),...e.components};return(0,o.jsxs)(o.Fragment,{children:[(0,o.jsxs)(n.p,{children:["The ",(0,o.jsx)(n.code,{children:"moon ci"})," command is a special command that should be ran in a continuous integration (CI)\nenvironment, as it does all the heavy lifting necessary for effectively running tasks."]}),"\n",(0,o.jsxs)(n.p,{children:["By default this will run all tasks that are affected by touched files and have the\n",(0,o.jsx)(n.a,{href:"../config/project#runinci",children:(0,o.jsx)(n.code,{children:"runInCI"})})," task option enabled."]}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{className:"language-shell",children:"$ moon ci\n"})}),"\n",(0,o.jsxs)(n.p,{children:["However, you can also provide a list of targets to explicitly run, which will still be filtered down\nby ",(0,o.jsx)(n.code,{children:"runInCI"}),"."]}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{className:"language-shell",children:"$ moon ci :build :lint\n"})}),"\n",(0,o.jsxs)(n.blockquote,{children:["\n",(0,o.jsxs)(n.p,{children:["View the official ",(0,o.jsx)(n.a,{href:"../guides/ci",children:"continuous integration guide"})," for a more in-depth example of how\nto utilize this command."]}),"\n"]}),"\n",(0,o.jsx)(n.h3,{id:"arguments",children:"Arguments"}),"\n",(0,o.jsxs)(n.ul,{children:["\n",(0,o.jsxs)(n.li,{children:[(0,o.jsx)(n.code,{children:"...[target]"})," - ",(0,o.jsx)(n.a,{href:"../concepts/target",children:"Targets"})," to run."]}),"\n"]}),"\n",(0,o.jsx)(n.h3,{id:"options",children:"Options"}),"\n",(0,o.jsxs)(n.ul,{children:["\n",(0,o.jsxs)(n.li,{children:[(0,o.jsx)(n.code,{children:"--base <rev>"})," - Base branch, commit, or revision to compare against\n(",(0,o.jsx)(n.a,{href:"../guides/ci#comparing-revisions",children:"learn more"}),")."]}),"\n",(0,o.jsxs)(n.li,{children:[(0,o.jsx)(n.code,{children:"--head <rev>"})," - Current branch, commit, or revision to compare with\n(",(0,o.jsx)(n.a,{href:"../guides/ci#comparing-revisions",children:"learn more"}),")."]}),"\n",(0,o.jsxs)(n.li,{children:[(0,o.jsx)(n.code,{children:"--job <index>"})," - Index of the current job."]}),"\n",(0,o.jsxs)(n.li,{children:[(0,o.jsx)(n.code,{children:"--jobTotal <total>"})," Total amount of jobs to run."]}),"\n"]})]})}function h(e={}){const{wrapper:n}={...(0,t.a)(),...e.components};return n?(0,o.jsx)(n,{...e,children:(0,o.jsx)(d,{...e})}):d(e)}},71670:(e,n,i)=>{i.d(n,{Z:()=>r,a:()=>c});var o=i(27378);const t={},s=o.createContext(t);function c(e){const n=o.useContext(s);return o.useMemo((function(){return"function"==typeof e?e(n):{...n,...e}}),[n,e])}function r(e){let n;return n=e.disableParentContext?"function"==typeof e.components?e.components(t):e.components||t:c(e.components),o.createElement(s.Provider,{value:n},e.children)}}}]);