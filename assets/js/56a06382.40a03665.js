"use strict";(self.webpackChunkwebsite=self.webpackChunkwebsite||[]).push([[15091],{97946:(e,n,t)=>{t.r(n),t.d(n,{assets:()=>p,contentTitle:()=>u,default:()=>g,frontMatter:()=>c,metadata:()=>d,toc:()=>h});var s=t(24246),r=t(71670),a=t(36642),l=(t(27457),t(33337)),i=t(39798),o=t(32189);const c={title:"Next example",sidebar_label:"Next"},u=void 0,d={id:"guides/examples/next",title:"Next example",description:"In this guide, you'll learn how to integrate Next.js into moon.",source:"@site/docs/guides/examples/next.mdx",sourceDirName:"guides/examples",slug:"/guides/examples/next",permalink:"/docs/guides/examples/next",draft:!1,unlisted:!1,editUrl:"https://github.com/moonrepo/moon/tree/master/website/docs/guides/examples/next.mdx",tags:[],version:"current",frontMatter:{title:"Next example",sidebar_label:"Next"},sidebar:"guides",previous:{title:"Nest",permalink:"/docs/guides/examples/nest"},next:{title:"Nuxt",permalink:"/docs/guides/examples/nuxt"}},p={},h=[{value:"Setup",id:"setup",level:2},{value:"ESLint integration",id:"eslint-integration",level:3},{value:"TypeScript integration",id:"typescript-integration",level:3},{value:"Configuration",id:"configuration",level:2},{value:"Root-level",id:"root-level",level:3},{value:"Project-level",id:"project-level",level:3}];function x(e){const n={a:"a",admonition:"admonition",blockquote:"blockquote",code:"code",em:"em",h2:"h2",h3:"h3",li:"li",p:"p",pre:"pre",ul:"ul",...(0,r.a)(),...e.components};return(0,s.jsxs)(s.Fragment,{children:[(0,s.jsx)(o.Z,{to:"https://github.com/moonrepo/examples/tree/master/apps/nextjs-app"}),"\n",(0,s.jsxs)(n.p,{children:["In this guide, you'll learn how to integrate ",(0,s.jsx)(n.a,{href:"https://nextjs.org",children:"Next.js"})," into moon."]}),"\n",(0,s.jsx)(n.p,{children:"Begin by creating a new Next.js project at a specified folder path (this should not be created in\nthe workspace root, unless a polyrepo)."}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-shell",children:"cd apps && npx create-next-app <project> --typescript\n"})}),"\n",(0,s.jsxs)(n.blockquote,{children:["\n",(0,s.jsxs)(n.p,{children:["View the ",(0,s.jsx)(n.a,{href:"https://nextjs.org/learn/basics/create-nextjs-app/setup",children:"official Next.js docs"})," for a\nmore in-depth guide to getting started!"]}),"\n"]}),"\n",(0,s.jsx)(n.h2,{id:"setup",children:"Setup"}),"\n",(0,s.jsxs)(n.p,{children:["Since Next.js is per-project, the associated moon tasks should be defined in each project's\n",(0,s.jsx)(n.a,{href:"../../config/project",children:(0,s.jsx)(n.code,{children:"moon.yml"})})," file."]}),"\n",(0,s.jsx)(n.admonition,{type:"tip",children:(0,s.jsxs)(n.p,{children:["We suggest inheriting Next.js tasks from the\n",(0,s.jsx)(n.a,{href:"https://github.com/moonrepo/moon-configs/tree/master/javascript/next",children:"official moon configuration preset"}),"."]})}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-yaml",metastring:'title="<project>/moon.yml"',children:"# Inherit tasks from the `next` preset\n# https://github.com/moonrepo/moon-configs\ntags: ['next']\n"})}),"\n",(0,s.jsx)(n.h3,{id:"eslint-integration",children:"ESLint integration"}),"\n",(0,s.jsxs)(n.p,{children:["Next.js has ",(0,s.jsx)(n.a,{href:"https://nextjs.org/docs/basic-features/eslint",children:"built-in support for ESLint"}),", which is\ngreat, but complicates things a bit. Because of this, you have two options for moving forward:"]}),"\n",(0,s.jsxs)(n.ul,{children:["\n",(0,s.jsxs)(n.li,{children:["Use a ",(0,s.jsxs)(n.a,{href:"./eslint",children:["global ",(0,s.jsx)(n.code,{children:"lint"})," task"]})," and bypass Next.js's solution (preferred)."]}),"\n",(0,s.jsx)(n.li,{children:"Use Next.js's solution only."}),"\n"]}),"\n",(0,s.jsxs)(n.p,{children:["Regardless of which option is chosen, the following changes are applicable to all options and should\nbe made. Begin be installing the\n",(0,s.jsx)(n.a,{href:"https://nextjs.org/docs/basic-features/eslint#eslint-config",children:(0,s.jsx)(n.code,{children:"eslint-config-next"})})," dependency in\nthe application's ",(0,s.jsx)(n.code,{children:"package.json"}),"."]}),"\n",(0,s.jsx)(a.Z,{dep:"eslint-config-next",package:"<project>",dev:!0}),"\n",(0,s.jsxs)(n.p,{children:["Since the Next.js app is located within a subfolder, we'll need to tell the ESLint plugin where to\nlocate it. This can be achieved with a project-level ",(0,s.jsx)(n.code,{children:".eslintrc.js"})," file."]}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-js",metastring:'title="<project>/.eslintrc.js"',children:"module.exports = {\n  extends: 'next', // or 'next/core-web-vitals'\n  settings: {\n    next: {\n      rootDir: __dirname,\n    },\n  },\n};\n"})}),"\n",(0,s.jsx)(n.p,{children:"With the basics now setup, choose the option that works best for you."}),"\n",(0,s.jsxs)(l.Z,{groupId:"lint-type",defaultValue:"global",values:[{label:"Global lint",value:"global"},{label:"Next.js lint",value:"nextjs"}],children:[(0,s.jsxs)(i.Z,{value:"global",children:[(0,s.jsxs)(n.p,{children:["We encourage using the global ",(0,s.jsx)(n.code,{children:"lint"})," task for consistency across all projects within the repository.\nWith this approach, the ",(0,s.jsx)(n.code,{children:"eslint"})," command itself will be ran and the ",(0,s.jsx)(n.code,{children:"next lint"})," command will be\nignored, but the ",(0,s.jsx)(n.code,{children:"eslint-config-next"})," rules will still be used."]}),(0,s.jsxs)(n.p,{children:["Additionally, we suggest disabling the linter during the build process, but is not a requirement. As\na potential alternative, add the ",(0,s.jsx)(n.code,{children:"lint"})," task as a dependency for the ",(0,s.jsx)(n.code,{children:"build"})," task."]}),(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-js",metastring:'title="<project>/next.config.js"',children:"module.exports = {\n  eslint: {\n    ignoreDuringBuilds: true,\n  },\n};\n"})})]}),(0,s.jsxs)(i.Z,{value:"nextjs",children:[(0,s.jsxs)(n.p,{children:["If you'd prefer to use the ",(0,s.jsx)(n.code,{children:"next lint"})," command, add it as a task to the project's\n",(0,s.jsx)(n.a,{href:"../../config/project",children:(0,s.jsx)(n.code,{children:"moon.yml"})}),"."]}),(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-yaml",metastring:'title="<project>/moon.yml"',children:"tasks:\n  lint:\n    command: 'next lint'\n    inputs:\n      - '@group(next)'\n"})}),(0,s.jsxs)(n.p,{children:["Furthermore, if a global ",(0,s.jsx)(n.code,{children:"lint"})," task exists, be sure to exclude it from being inherited."]}),(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-yaml",metastring:'title="<project>/moon.yml"',children:"workspace:\n  inheritedTasks:\n    exclude: ['lint']\n"})})]})]}),"\n",(0,s.jsx)(n.h3,{id:"typescript-integration",children:"TypeScript integration"}),"\n",(0,s.jsxs)(n.p,{children:["Next.js also has\n",(0,s.jsx)(n.a,{href:"https://nextjs.org/docs/basic-features/typescript",children:"built-in support for TypeScript"}),", but has\nsimilar caveats to the ",(0,s.jsx)(n.a,{href:"#eslint-integration",children:"ESLint integration"}),". TypeScript itself is a bit\ninvolved, so we suggest reading the official Next.js documentation before continuing."]}),"\n",(0,s.jsxs)(n.p,{children:["At this point we'll assume that a ",(0,s.jsx)(n.code,{children:"tsconfig.json"})," has been created in the application, and\ntypechecking works. From here we suggest utilizing a ",(0,s.jsxs)(n.a,{href:"./typescript",children:["global ",(0,s.jsx)(n.code,{children:"typecheck"})," task"]})," for\nconsistency across all projects within the repository."]}),"\n",(0,s.jsxs)(n.p,{children:["Additionally, we suggest disabling the typechecker during the build process, but is not a\nrequirement. As a potential alternative, add the ",(0,s.jsx)(n.code,{children:"typecheck"})," task as a dependency for the ",(0,s.jsx)(n.code,{children:"build"}),"\ntask."]}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-js",metastring:'title="<project>/next.config.js"',children:"module.exports = {\n  typescript: {\n    ignoreBuildErrors: true,\n  },\n};\n"})}),"\n",(0,s.jsx)(n.h2,{id:"configuration",children:"Configuration"}),"\n",(0,s.jsx)(n.h3,{id:"root-level",children:"Root-level"}),"\n",(0,s.jsxs)(n.p,{children:["We suggest ",(0,s.jsx)(n.em,{children:"against"})," root-level configuration, as Next.js should be installed per-project, and the\n",(0,s.jsx)(n.code,{children:"next"})," command expects the configuration to live relative to the project root."]}),"\n",(0,s.jsx)(n.h3,{id:"project-level",children:"Project-level"}),"\n",(0,s.jsxs)(n.p,{children:["When creating a new Next.js project, a\n",(0,s.jsx)(n.a,{href:"https://nextjs.org/docs/api-reference/next.config.js/introduction",children:(0,s.jsx)(n.code,{children:"next.config.<js|mjs>"})})," is\ncreated, and ",(0,s.jsx)(n.em,{children:"must"})," exist in the project root. This allows each project to configure Next.js for\ntheir needs."]}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-js",metastring:'title="<project>/next.config.js"',children:"module.exports = {\n  compress: true,\n};\n"})})]})}function g(e={}){const{wrapper:n}={...(0,r.a)(),...e.components};return n?(0,s.jsx)(n,{...e,children:(0,s.jsx)(x,{...e})}):x(e)}},39798:(e,n,t)=>{t.d(n,{Z:()=>l});t(27378);var s=t(40624);const r={tabItem:"tabItem_wHwb"};var a=t(24246);function l(e){let{children:n,hidden:t,className:l}=e;return(0,a.jsx)("div",{role:"tabpanel",className:(0,s.Z)(r.tabItem,l),hidden:t,children:n})}},33337:(e,n,t)=>{t.d(n,{Z:()=>h});var s=t(27378),r=t(40624),a=t(83457),l=t(35595),i=t(76457);const o={tabList:"tabList_J5MA",tabItem:"tabItem_l0OV"};var c=t(24246);function u(e){let{className:n,block:t,selectedValue:s,selectValue:l,tabValues:i}=e;const u=[],{blockElementScrollPositionUntilNextRender:d}=(0,a.o5)(),p=e=>{const n=e.currentTarget,t=u.indexOf(n),r=i[t].value;r!==s&&(d(n),l(r))},h=e=>{let n=null;switch(e.key){case"Enter":p(e);break;case"ArrowRight":{const t=u.indexOf(e.currentTarget)+1;n=u[t]??u[0];break}case"ArrowLeft":{const t=u.indexOf(e.currentTarget)-1;n=u[t]??u[u.length-1];break}}n?.focus()};return(0,c.jsx)("ul",{role:"tablist","aria-orientation":"horizontal",className:(0,r.Z)("tabs",{"tabs--block":t},n),children:i.map((e=>{let{value:n,label:t,attributes:a}=e;return(0,c.jsx)("li",{role:"tab",tabIndex:s===n?0:-1,"aria-selected":s===n,ref:e=>u.push(e),onKeyDown:h,onClick:p,...a,className:(0,r.Z)("tabs__item",o.tabItem,a?.className,{"tabs__item--active":s===n}),children:t??n},n)}))})}function d(e){let{lazy:n,children:t,selectedValue:r}=e;const a=(Array.isArray(t)?t:[t]).filter(Boolean);if(n){const e=a.find((e=>e.props.value===r));return e?(0,s.cloneElement)(e,{className:"margin-top--md"}):null}return(0,c.jsx)("div",{className:"margin-top--md",children:a.map(((e,n)=>(0,s.cloneElement)(e,{key:n,hidden:e.props.value!==r})))})}function p(e){const n=(0,l.Y)(e);return(0,c.jsxs)("div",{className:(0,r.Z)("tabs-container",o.tabList),children:[(0,c.jsx)(u,{...n,...e}),(0,c.jsx)(d,{...n,...e})]})}function h(e){const n=(0,i.Z)();return(0,c.jsx)(p,{...e,children:(0,l.h)(e.children)},String(n))}},35595:(e,n,t)=>{t.d(n,{Y:()=>h,h:()=>c});var s=t(27378),r=t(3620),a=t(9834),l=t(30654),i=t(70784),o=t(55643);function c(e){return s.Children.toArray(e).filter((e=>"\n"!==e)).map((e=>{if(!e||(0,s.isValidElement)(e)&&function(e){const{props:n}=e;return!!n&&"object"==typeof n&&"value"in n}(e))return e;throw new Error(`Docusaurus error: Bad <Tabs> child <${"string"==typeof e.type?e.type:e.type.name}>: all children of the <Tabs> component should be <TabItem>, and every <TabItem> should have a unique "value" prop.`)}))?.filter(Boolean)??[]}function u(e){const{values:n,children:t}=e;return(0,s.useMemo)((()=>{const e=n??function(e){return c(e).map((e=>{let{props:{value:n,label:t,attributes:s,default:r}}=e;return{value:n,label:t,attributes:s,default:r}}))}(t);return function(e){const n=(0,i.l)(e,((e,n)=>e.value===n.value));if(n.length>0)throw new Error(`Docusaurus error: Duplicate values "${n.map((e=>e.value)).join(", ")}" found in <Tabs>. Every value needs to be unique.`)}(e),e}),[n,t])}function d(e){let{value:n,tabValues:t}=e;return t.some((e=>e.value===n))}function p(e){let{queryString:n=!1,groupId:t}=e;const a=(0,r.k6)(),i=function(e){let{queryString:n=!1,groupId:t}=e;if("string"==typeof n)return n;if(!1===n)return null;if(!0===n&&!t)throw new Error('Docusaurus error: The <Tabs> component groupId prop is required if queryString=true, because this value is used as the search param name. You can also provide an explicit value such as queryString="my-search-param".');return t??null}({queryString:n,groupId:t});return[(0,l._X)(i),(0,s.useCallback)((e=>{if(!i)return;const n=new URLSearchParams(a.location.search);n.set(i,e),a.replace({...a.location,search:n.toString()})}),[i,a])]}function h(e){const{defaultValue:n,queryString:t=!1,groupId:r}=e,l=u(e),[i,c]=(0,s.useState)((()=>function(e){let{defaultValue:n,tabValues:t}=e;if(0===t.length)throw new Error("Docusaurus error: the <Tabs> component requires at least one <TabItem> children component");if(n){if(!d({value:n,tabValues:t}))throw new Error(`Docusaurus error: The <Tabs> has a defaultValue "${n}" but none of its children has the corresponding value. Available values are: ${t.map((e=>e.value)).join(", ")}. If you intend to show no default tab, use defaultValue={null} instead.`);return n}const s=t.find((e=>e.default))??t[0];if(!s)throw new Error("Unexpected error: 0 tabValues");return s.value}({defaultValue:n,tabValues:l}))),[h,x]=p({queryString:t,groupId:r}),[g,j]=function(e){let{groupId:n}=e;const t=function(e){return e?`docusaurus.tab.${e}`:null}(n),[r,a]=(0,o.Nk)(t);return[r,(0,s.useCallback)((e=>{t&&a.set(e)}),[t,a])]}({groupId:r}),m=(()=>{const e=h??g;return d({value:e,tabValues:l})?e:null})();(0,a.Z)((()=>{m&&c(m)}),[m]);return{selectedValue:i,selectValue:(0,s.useCallback)((e=>{if(!d({value:e,tabValues:l}))throw new Error(`Can't select invalid tab value=${e}`);c(e),x(e),j(e)}),[x,j,l]),tabValues:l}}},36642:(e,n,t)=>{t.d(n,{Z:()=>d});var s=t(52807),r=t(39798),a=t(33337),l=t(24246);function i(e,n,t){let s=e.package?`yarn workspace ${e.package} add `:"yarn add ";return e.dev?s+="--dev ":e.peer&&(s+="--peer "),t&&n&&!e.package&&(s+="-W "),s+=e.dep,s}function o(e){let n="npm install ";return e.dev?n+="--save-dev ":e.peer&&(n+="--save-peer "),e.package&&(n+=`--workspace ${e.package} `),n+=e.dep,n}function c(e,n){let t="pnpm add ";return e.dev?t+="--save-dev ":e.peer&&(t+="--save-peer "),e.package?t+=`--filter ${e.package} `:n&&(t+="-w "),t+=e.dep,t}function u(e){let n="bun install ";return e.dev?n+="--dev ":e.peer&&(n+="--peer "),n+=e.dep,n}function d(e){let n=i(e,!1,!0),t=c(e,!1);return e.package||(n+="\n\n# If using workspaces\n",t+="\n\n# If using workspaces\n",n+=i(e,!0,!0),t+=c(e,!0)),(0,l.jsxs)(a.Z,{groupId:"package-manager",defaultValue:"yarn",values:[{label:"Yarn",value:"yarn"},{label:"Yarn (classic)",value:"yarn1"},{label:"npm",value:"npm"},{label:"pnpm",value:"pnpm"},{label:"Bun",value:"bun"}],children:[(0,l.jsx)(r.Z,{value:"yarn",children:(0,l.jsx)(s.default,{language:"shell",children:i(e,!1,!1)})}),(0,l.jsx)(r.Z,{value:"yarn1",children:(0,l.jsx)(s.default,{language:"shell",children:n})}),(0,l.jsx)(r.Z,{value:"npm",children:(0,l.jsx)(s.default,{language:"shell",children:o(e)})}),(0,l.jsx)(r.Z,{value:"pnpm",children:(0,l.jsx)(s.default,{language:"shell",children:t})}),(0,l.jsx)(r.Z,{value:"bun",children:(0,l.jsx)(s.default,{language:"shell",children:u(e)})})]})}},27457:(e,n,t)=>{t.d(n,{Z:()=>o});var s=t(52807),r=t(39798),a=t(33337),l=t(24246);function i(e,n,t){return`${e} create ${n} ${t.join(" ")}`.trim()}function o(e){let{dep:n,args:t=[]}=e;return(0,l.jsxs)(a.Z,{groupId:"package-manager",defaultValue:"yarn",values:[{label:"Yarn",value:"yarn"},{label:"Yarn (classic)",value:"yarn1"},{label:"npm",value:"npm"},{label:"pnpm",value:"pnpm"}],children:[(0,l.jsx)(r.Z,{value:"yarn",children:(0,l.jsx)(s.default,{language:"shell",children:i("yarn",n,t)})}),(0,l.jsx)(r.Z,{value:"yarn1",children:(0,l.jsx)(s.default,{language:"shell",children:i("yarn",n,t)})}),(0,l.jsx)(r.Z,{value:"npm",children:(0,l.jsx)(s.default,{language:"shell",children:i("npm",n,t)})}),(0,l.jsx)(r.Z,{value:"pnpm",children:(0,l.jsx)(s.default,{language:"shell",children:i("pnpm",n,t)})})]})}},32189:(e,n,t)=>{t.d(n,{Z:()=>l});var s=t(83469),r=t(31792),a=t(24246);function l(e){let{to:n}=e;return(0,a.jsx)("a",{href:n,target:"_blank",className:"float-right inline-block",style:{marginTop:"-3em"},children:(0,a.jsx)(r.Z,{icon:s.dT$})})}},71670:(e,n,t)=>{t.d(n,{Z:()=>i,a:()=>l});var s=t(27378);const r={},a=s.createContext(r);function l(e){const n=s.useContext(a);return s.useMemo((function(){return"function"==typeof e?e(n):{...n,...e}}),[n,e])}function i(e){let n;return n=e.disableParentContext?"function"==typeof e.components?e.components(r):e.components||r:l(e.components),s.createElement(a.Provider,{value:n},e.children)}}}]);