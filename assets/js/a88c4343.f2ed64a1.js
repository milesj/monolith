"use strict";(self.webpackChunkwebsite=self.webpackChunkwebsite||[]).push([[68282],{51897:(e,n,t)=>{t.r(n),t.d(n,{assets:()=>a,contentTitle:()=>i,default:()=>p,frontMatter:()=>c,metadata:()=>l,toc:()=>d});var s=t(24246),r=t(71670),o=t(79022);const c={title:"query projects",sidebar_label:"projects"},i=void 0,l={id:"commands/query/projects",title:"query projects",description:"Use the moon query projects sub-command to query information about all projects in the project",source:"@site/docs/commands/query/projects.mdx",sourceDirName:"commands/query",slug:"/commands/query/projects",permalink:"/docs/commands/query/projects",draft:!1,unlisted:!1,editUrl:"https://github.com/moonrepo/moon/tree/master/website/docs/commands/query/projects.mdx",tags:[],version:"current",frontMatter:{title:"query projects",sidebar_label:"projects"},sidebar:"docs",previous:{title:"hash-diff",permalink:"/docs/commands/query/hash-diff"},next:{title:"tasks",permalink:"/docs/commands/query/tasks"}},a={},d=[{value:"Affected projects",id:"affected-projects",level:3},{value:"Arguments",id:"arguments",level:3},{value:"Options",id:"options",level:3},{value:"Filters",id:"filters",level:4},{value:"Configuration",id:"configuration",level:3}];function h(e){const n={a:"a",code:"code",em:"em",h3:"h3",h4:"h4",li:"li",p:"p",pre:"pre",ul:"ul",...(0,r.a)(),...e.components};return(0,s.jsxs)(s.Fragment,{children:[(0,s.jsxs)(n.p,{children:["Use the ",(0,s.jsx)(n.code,{children:"moon query projects"})," sub-command to query information about all projects in the project\ngraph. The project list can be filtered by passing a ",(0,s.jsx)(n.a,{href:"../../concepts/query-lang",children:"query statement"})," as\nan argument, or by using ",(0,s.jsx)(n.a,{href:"#options",children:"options"})," arguments."]}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-shell",children:'# Find all projects\n$ moon query projects\n\n# Find all projects with an id that matches "react"\n$ moon query projects --id react\n$ moon query projects "project~react"\n\n# Find all projects with a `lint` or `build` task\n$ moon query projects --tasks "lint|build"\n$ moon query projects "task=[lint,build]"\n'})}),"\n",(0,s.jsxs)(n.p,{children:["By default, this will output a list of projects in the format of\n",(0,s.jsx)(n.code,{children:"<id> | <source> | <stack> | <type> | <language>"}),", separated by new lines."]}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{children:"web | apps/web | frontend | application | typescript\n"})}),"\n",(0,s.jsxs)(n.p,{children:["The projects can also be output in JSON (",(0,s.jsx)(n.a,{href:"/api/types/interface/Project",children:"which contains all data"}),") by\npassing the ",(0,s.jsx)(n.code,{children:"--json"})," flag. The output has the following structure:"]}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-ts",children:"{\n\tprojects: Project[],\n\toptions: QueryOptions,\n}\n"})}),"\n",(0,s.jsx)(n.h3,{id:"affected-projects",children:"Affected projects"}),"\n",(0,s.jsxs)(n.p,{children:["This command can also be used to query for affected projects, based on the state of the VCS working\ntree. For advanced control, you can also pass the results of ",(0,s.jsx)(n.code,{children:"moon query touched-files"})," to stdin."]}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-shell",children:"# Find all affected projects\n$ moon query projects --affected\n\n# Find all affected projects using the results of another query\n$ moon query touched-files | moon query projects --affected\n"})}),"\n",(0,s.jsx)(n.h3,{id:"arguments",children:"Arguments"}),"\n",(0,s.jsxs)(n.ul,{children:["\n",(0,s.jsxs)(n.li,{children:[(0,s.jsx)(n.code,{children:"[query]"})," - An optional ",(0,s.jsx)(n.a,{href:"../../concepts/query-lang",children:"query statement"})," to filter projects with. When\nprovided, all ",(0,s.jsx)(n.a,{href:"#filters",children:"filter options"})," are ignored. ",(0,s.jsx)(o.Z,{version:"1.4.0"})]}),"\n"]}),"\n",(0,s.jsx)(n.h3,{id:"options",children:"Options"}),"\n",(0,s.jsxs)(n.ul,{children:["\n",(0,s.jsxs)(n.li,{children:[(0,s.jsx)(n.code,{children:"--affected"})," - Filter projects that have been affected by touched files. This will only filter\nbased on files, and ",(0,s.jsx)(n.em,{children:"does not"})," include upstream or downstream dependencies."]}),"\n",(0,s.jsxs)(n.li,{children:[(0,s.jsx)(n.code,{children:"--dependents"})," - Include direct dependents of queried projects."]}),"\n",(0,s.jsxs)(n.li,{children:[(0,s.jsx)(n.code,{children:"--json"})," - Display the projects in JSON format."]}),"\n"]}),"\n",(0,s.jsx)(n.h4,{id:"filters",children:"Filters"}),"\n",(0,s.jsx)(n.p,{children:"All option values are case-insensitive regex patterns."}),"\n",(0,s.jsxs)(n.ul,{children:["\n",(0,s.jsxs)(n.li,{children:[(0,s.jsx)(n.code,{children:"--alias <regex>"})," - Filter projects that match this alias."]}),"\n",(0,s.jsxs)(n.li,{children:[(0,s.jsx)(n.code,{children:"--id <regex>"})," - Filter projects that match this ID/name."]}),"\n",(0,s.jsxs)(n.li,{children:[(0,s.jsx)(n.code,{children:"--language <regex>"})," - Filter projects of this programming language."]}),"\n",(0,s.jsxs)(n.li,{children:[(0,s.jsx)(n.code,{children:"--source <regex>"})," - Filter projects that match this source path."]}),"\n",(0,s.jsxs)(n.li,{children:[(0,s.jsx)(n.code,{children:"--stack <regex>"})," - Filter projects of the tech stack."]}),"\n",(0,s.jsxs)(n.li,{children:[(0,s.jsx)(n.code,{children:"--tags <regex>"})," - Filter projects that have the following tags."]}),"\n",(0,s.jsxs)(n.li,{children:[(0,s.jsx)(n.code,{children:"--tasks <regex>"})," - Filter projects that have the following tasks."]}),"\n",(0,s.jsxs)(n.li,{children:[(0,s.jsx)(n.code,{children:"--type <regex>"})," - Filter project of this type."]}),"\n"]}),"\n",(0,s.jsx)(n.h3,{id:"configuration",children:"Configuration"}),"\n",(0,s.jsxs)(n.ul,{children:["\n",(0,s.jsxs)(n.li,{children:[(0,s.jsx)(n.a,{href:"../../config/workspace#projects",children:(0,s.jsx)(n.code,{children:"projects"})})," in ",(0,s.jsx)(n.code,{children:".moon/workspace.yml"})]}),"\n"]})]})}function p(e={}){const{wrapper:n}={...(0,r.a)(),...e.components};return n?(0,s.jsx)(n,{...e,children:(0,s.jsx)(h,{...e})}):h(e)}},79022:(e,n,t)=>{t.d(n,{Z:()=>o});var s=t(9619),r=t(24246);function o(e){let{header:n,inline:t,updated:o,version:c}=e;return(0,r.jsx)(s.Z,{text:`v${c}`,variant:o?"success":"info",className:n?"absolute right-0 top-1.5":t?"inline-block":"ml-2"})}},9619:(e,n,t)=>{t.d(n,{Z:()=>i});var s=t(40624),r=t(31792),o=t(24246);const c={failure:"bg-red-100 text-red-900",info:"bg-pink-100 text-pink-900",success:"bg-green-100 text-green-900",warning:"bg-orange-100 text-orange-900"};function i(e){let{className:n,icon:t,text:i,variant:l}=e;return(0,o.jsxs)("span",{className:(0,s.Z)("inline-flex items-center px-1 py-0.5 rounded text-xs font-bold uppercase",l?c[l]:"bg-gray-100 text-gray-800",n),children:[t&&(0,o.jsx)(r.Z,{icon:t,className:"mr-1"}),i]})}},71670:(e,n,t)=>{t.d(n,{Z:()=>i,a:()=>c});var s=t(27378);const r={},o=s.createContext(r);function c(e){const n=s.useContext(o);return s.useMemo((function(){return"function"==typeof e?e(n):{...n,...e}}),[n,e])}function i(e){let n;return n=e.disableParentContext?"function"==typeof e.components?e.components(r):e.components||r:c(e.components),s.createElement(o.Provider,{value:n},e.children)}}}]);