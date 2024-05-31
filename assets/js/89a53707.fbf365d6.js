"use strict";(self.webpackChunkwebsite=self.webpackChunkwebsite||[]).push([[23157],{92469:(e,n,o)=>{o.r(n),o.d(n,{assets:()=>d,contentTitle:()=>l,default:()=>p,frontMatter:()=>t,metadata:()=>c,toc:()=>u});var s=o(24246),r=o(71670),a=o(33337),i=o(39798);const t={title:"Docker usage"},l=void 0,c={id:"guides/docker",title:"Docker usage",description:"Using Docker to run your applications? Or build your artifacts? No",source:"@site/docs/guides/docker.mdx",sourceDirName:"guides",slug:"/guides/docker",permalink:"/docs/guides/docker",draft:!1,unlisted:!1,editUrl:"https://github.com/moonrepo/moon/tree/master/website/docs/guides/docker.mdx",tags:[],version:"current",frontMatter:{title:"Docker usage"},sidebar:"guides",previous:{title:"Debugging a task",permalink:"/docs/guides/debug-task"},next:{title:"Extensions",permalink:"/docs/guides/extensions"}},d={},u=[{value:"Requirements",id:"requirements",level:2},{value:"<code>Dockerfile</code> setup",id:"dockerfile-setup",level:2},{value:"What we&#39;re trying to avoid",id:"what-were-trying-to-avoid",level:3},{value:"Scaffolding the bare minimum",id:"scaffolding-the-bare-minimum",level:3},{value:"Copying necessary source files",id:"copying-necessary-source-files",level:3},{value:"Pruning extraneous files",id:"pruning-extraneous-files",level:3},{value:"Final result",id:"final-result",level:3},{value:"Troubleshooting",id:"troubleshooting",level:2},{value:"Supporting <code>node:alpine</code> images",id:"supporting-nodealpine-images",level:3}];function h(e){const n={a:"a",admonition:"admonition",code:"code",em:"em",h2:"h2",h3:"h3",li:"li",ol:"ol",p:"p",pre:"pre",ul:"ul",...(0,r.a)(),...e.components};return(0,s.jsxs)(s.Fragment,{children:[(0,s.jsxs)(n.p,{children:["Using ",(0,s.jsx)(n.a,{href:"https://www.docker.com/",children:"Docker"})," to run your applications? Or build your artifacts? No\nworries, moon can be utilized with Docker, and supports a robust integration layer."]}),"\n",(0,s.jsx)(n.admonition,{type:"success",children:(0,s.jsxs)(n.p,{children:["Looking to speed up your Docker builds? Want to build in the cloud?\n",(0,s.jsx)(n.a,{href:"https://depot.dev?ref=moonrepo",children:"Give Depot a try"}),"!"]})}),"\n",(0,s.jsx)(n.h2,{id:"requirements",children:"Requirements"}),"\n",(0,s.jsxs)(n.p,{children:["The first requirement, which is very important, is adding ",(0,s.jsx)(n.code,{children:".moon/cache"})," to the workspace root\n",(0,s.jsx)(n.code,{children:".dockerignore"})," (moon assumes builds are running from the root). Not all files in ",(0,s.jsx)(n.code,{children:".moon/cache"})," are\nportable across machines/environments, so copying these file into Docker will definitely cause\ninteroperability issues."]}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-text",metastring:'title=".dockerignore"',children:".moon/cache\n"})}),"\n",(0,s.jsxs)(n.p,{children:["The other requirement depends on how you want to integrate Git with Docker. Since moon executes\n",(0,s.jsx)(n.code,{children:"git"})," commands under the hood, there are some special considerations to be aware of when running\nmoon within Docker. There's 2 scenarios to choose from:"]}),"\n",(0,s.jsxs)(n.ol,{children:["\n",(0,s.jsxs)(n.li,{children:["(recommended) Add the ",(0,s.jsx)(n.code,{children:".git"})," folder to ",(0,s.jsx)(n.code,{children:".dockerignore"}),", so that it's not ",(0,s.jsx)(n.code,{children:"COPY"}),"'d. moon will\ncontinue to work just fine, albeit with some functionality disabled, like caching."]}),"\n",(0,s.jsxs)(n.li,{children:["Ensure that the ",(0,s.jsx)(n.code,{children:"git"})," library is installed in the container, and copy the ",(0,s.jsx)(n.code,{children:".git"})," folder with\n",(0,s.jsx)(n.code,{children:"COPY"}),". moon will work with full functionality, but it will increase the overall size of the\nimage because of caching."]}),"\n"]}),"\n",(0,s.jsxs)(n.h2,{id:"dockerfile-setup",children:[(0,s.jsx)(n.code,{children:"Dockerfile"})," setup"]}),"\n",(0,s.jsxs)(n.p,{children:["We're very familiar with how tedious ",(0,s.jsx)(n.code,{children:"Dockerfile"}),"s are to write and maintain, so in an effort to\nreduce this headache, we've built a handful of tools to make this process much easier. With moon,\nwe'll take advantage of Docker's layer caching and staged builds as much as possible."]}),"\n",(0,s.jsx)(n.p,{children:"With that being said, there's many approaches you can utilize, depending on your workflow (we'll\ndocument them below):"}),"\n",(0,s.jsxs)(n.ul,{children:["\n",(0,s.jsxs)(n.li,{children:["Running ",(0,s.jsx)(n.code,{children:"moon docker"})," commands ",(0,s.jsx)(n.em,{children:"before"})," running ",(0,s.jsx)(n.code,{children:"docker run|build"})," commands."]}),"\n",(0,s.jsxs)(n.li,{children:["Running ",(0,s.jsx)(n.code,{children:"moon docker"})," commands ",(0,s.jsx)(n.em,{children:"within"})," the ",(0,s.jsx)(n.code,{children:"Dockerfile"}),"."]}),"\n",(0,s.jsx)(n.li,{children:"Using multi-staged or standard builds."}),"\n",(0,s.jsx)(n.li,{children:"Something else unique to your setup!"}),"\n"]}),"\n",(0,s.jsx)(n.h3,{id:"what-were-trying-to-avoid",children:"What we're trying to avoid"}),"\n",(0,s.jsxs)(n.p,{children:["Before we dive into writing a perfect ",(0,s.jsx)(n.code,{children:"Dockerfile"}),", we'll briefly talk about the pain points we're\ntrying to avoid. In the context of Node.js and monorepo's, you may be familiar with having to ",(0,s.jsx)(n.code,{children:"COPY"}),"\neach individual ",(0,s.jsx)(n.code,{children:"package.json"})," in the monorepo before installing ",(0,s.jsx)(n.code,{children:"node_modules"}),", to effectively use\nlayer caching. This is very brittle, as each new application or package is created, every\n",(0,s.jsx)(n.code,{children:"Dockerfile"})," in the monorepo will need to be modified to account for this new ",(0,s.jsx)(n.code,{children:"package.json"}),"."]}),"\n",(0,s.jsxs)(n.p,{children:["Furthermore, we'll have to follow a similar process for ",(0,s.jsx)(n.em,{children:"only copying source files"})," necessary for\nthe build or ",(0,s.jsx)(n.code,{children:"CMD"})," to complete. This is ",(0,s.jsx)(n.em,{children:"very tedious"}),", so most developers simply use ",(0,s.jsx)(n.code,{children:"COPY . ."})," and\nforget about it. Copying the entire monorepo is costly, especially as it grows."]}),"\n",(0,s.jsxs)(n.p,{children:["As an example, we'll use moon's official repository. The ",(0,s.jsx)(n.code,{children:"Dockerfile"})," would look something like the\nfollowing."]}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-docker",children:"FROM node:latest\n\nWORKDIR /app\n\n# Install moon binary\nRUN npm install -g @moonrepo/cli\n\n# Copy moon files\nCOPY ./.moon ./.moon\n\n# Copy all package.json's and lockfiles\nCOPY ./packages/cli/package.json ./packages/cli/package.json\nCOPY ./packages/core-linux-arm64-gnu/package.json ./packages/core-linux-arm64-gnu/package.json\nCOPY ./packages/core-linux-arm64-musl/package.json ./packages/core-linux-arm64-musl/package.json\nCOPY ./packages/core-linux-x64-gnu/package.json ./packages/core-linux-x64-gnu/package.json\nCOPY ./packages/core-linux-x64-musl/package.json ./packages/core-linux-x64-musl/package.json\nCOPY ./packages/core-macos-arm64/package.json ./packages/core-macos-arm64/package.json\nCOPY ./packages/core-macos-x64/package.json ./packages/core-macos-x64/package.json\nCOPY ./packages/core-windows-x64-msvc/package.json ./packages/core-windows-x64-msvc/package.json\nCOPY ./packages/runtime/package.json ./packages/runtime/package.json\nCOPY ./packages/types/package.json ./packages/types/package.json\nCOPY ./package.json ./package.json\nCOPY ./yarn.lock ./yarn.lock\nCOPY ./.yarn ./.yarn\nCOPY ./.yarnrc.yml ./yarnrc.yml\n\n# Install toolchain and dependencies\nRUN moon docker setup\n\n# Copy project and required files\nCOPY ./packages/types ./packages/types\nCOPY ./packages/runtime ./packages/runtime\n# OR COPY . .\n\n# Build the target\nRUN moon run runtime:build\n"})}),"\n",(0,s.jsx)(n.p,{children:"For such a small monorepo, this already looks too confusing!!! Let's remedy this by utilizing moon\nitself to the fullest!"}),"\n",(0,s.jsx)(n.h3,{id:"scaffolding-the-bare-minimum",children:"Scaffolding the bare minimum"}),"\n",(0,s.jsxs)(n.p,{children:["The first step in this process is to only copy the bare minimum of files necessary for installing\ndependencies (Node.js modules, etc). This is typically manifests (",(0,s.jsx)(n.code,{children:"package.json"}),"), lockfiles\n(",(0,s.jsx)(n.code,{children:"yarn.lock"}),", etc), and any configuration (",(0,s.jsx)(n.code,{children:".yarnrc.yml"}),", etc)."]}),"\n",(0,s.jsxs)(n.p,{children:["This can all be achieved by the ",(0,s.jsx)(n.a,{href:"../commands/docker/scaffold",children:(0,s.jsx)(n.code,{children:"moon docker scaffold"})})," command, which scaffolds a\nskeleton of the repository structure, with only necessary files (the above). Let's update our\n",(0,s.jsx)(n.code,{children:"Dockerfile"})," usage."]}),"\n",(0,s.jsxs)(a.Z,{groupId:"dockerfile",defaultValue:"standard",values:[{label:"Standard",value:"standard"},{label:"Multi-staged",value:"staged"}],children:[(0,s.jsxs)(i.Z,{value:"standard",children:[(0,s.jsxs)(n.p,{children:["This assumes ",(0,s.jsx)(n.a,{href:"../commands/docker/scaffold",children:(0,s.jsx)(n.code,{children:"moon docker scaffold <project>"})})," is ran outside of the ",(0,s.jsx)(n.code,{children:"Dockerfile"}),"."]}),(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-docker",children:"FROM node:latest\nWORKDIR /app\n\n# Install moon binary\nRUN npm install -g @moonrepo/cli\n\n# Copy workspace skeleton\nCOPY ./.moon/docker/workspace .\n\n# Install toolchain and dependencies\nRUN moon docker setup\n"})})]}),(0,s.jsx)(i.Z,{value:"staged",children:(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-docker",children:"#### BASE\nFROM node:latest AS base\nWORKDIR /app\n\n# Install moon binary\nRUN npm install -g @moonrepo/cli\n\n#### WORKSPACE\nFROM base AS workspace\nWORKDIR /app\n\n# Copy entire repository and scaffold\nCOPY . .\nRUN moon docker scaffold <project>\n\n#### BUILD\nFROM base AS build\nWORKDIR /app\n\n# Copy workspace skeleton\nCOPY --from=workspace /app/.moon/docker/workspace .\n\n# Install toolchain and dependencies\nRUN moon docker setup\n"})})})]}),"\n",(0,s.jsx)(n.p,{children:"And with this, our dependencies will be layer cached effectively! Let's now move onto copying source\nfiles."}),"\n",(0,s.jsx)(n.h3,{id:"copying-necessary-source-files",children:"Copying necessary source files"}),"\n",(0,s.jsxs)(n.p,{children:["The next step is to copy all source files necessary for ",(0,s.jsx)(n.code,{children:"CMD"})," or any ",(0,s.jsx)(n.code,{children:"RUN"})," commands to execute\ncorrectly. This typically requires copying all source files for the project ",(0,s.jsx)(n.em,{children:"and"})," all source files\nof the project's dependencies... NOT the entire repository!"]}),"\n",(0,s.jsxs)(n.p,{children:["Luckily our ",(0,s.jsx)(n.a,{href:"../commands/docker/scaffold",children:(0,s.jsx)(n.code,{children:"moon docker scaffold <project>"})})," command has already done this for us! Let's\ncontinue updating our ",(0,s.jsx)(n.code,{children:"Dockerfile"})," to account for this, by appending the following:"]}),"\n",(0,s.jsxs)(a.Z,{groupId:"dockerfile",defaultValue:"standard",values:[{label:"Standard",value:"standard"},{label:"Multi-staged",value:"staged"}],children:[(0,s.jsx)(i.Z,{value:"standard",children:(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-docker",children:"# Copy source files\nCOPY ./.moon/docker/sources .\n\n# Run something\nRUN moon run <project>:<task>\n"})})}),(0,s.jsx)(i.Z,{value:"staged",children:(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-docker",children:"# Copy source files\nCOPY --from=workspace /app/.moon/docker/sources .\n\n# Run something\nRUN moon run <project>:<task>\n"})})})]}),"\n",(0,s.jsx)(n.admonition,{type:"info",children:(0,s.jsxs)(n.p,{children:["If you need additional files for your commands to run successfully, you can manually use ",(0,s.jsx)(n.code,{children:"COPY"})," or\npass ",(0,s.jsx)(n.code,{children:"--include"})," to the scaffold command."]})}),"\n",(0,s.jsx)(n.h3,{id:"pruning-extraneous-files",children:"Pruning extraneous files"}),"\n",(0,s.jsxs)(n.p,{children:["Now that we've ran a command or built an artifact, we should prune the Docker environment to remove\nunneeded files and folders. We can do this with the ",(0,s.jsx)(n.a,{href:"../commands/docker/prune",children:(0,s.jsx)(n.code,{children:"moon docker prune"})})," command, which\n",(0,s.jsx)(n.em,{children:"must be ran"})," within the context of a ",(0,s.jsx)(n.code,{children:"Dockerfile"}),"!"]}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-docker",children:"# Prune workspace\nRUN moon docker prune\n"})}),"\n",(0,s.jsx)(n.p,{children:"When ran, this command will do the following:"}),"\n",(0,s.jsxs)(n.ul,{children:["\n",(0,s.jsx)(n.li,{children:"Install production only dependencies for the projects that were scaffolded."}),"\n",(0,s.jsxs)(n.li,{children:["Remove extraneous dependencies (",(0,s.jsx)(n.code,{children:"node_modules"}),") for unfocused projects."]}),"\n"]}),"\n",(0,s.jsx)(n.h3,{id:"final-result",children:"Final result"}),"\n",(0,s.jsxs)(n.p,{children:["And with this moon integration, we've reduced the original ",(0,s.jsx)(n.code,{children:"Dockerfile"})," of 35 lines to 18 lines, a\nreduction of almost 50%. The original file can also be seen as ",(0,s.jsx)(n.code,{children:"O(n)"}),", as each new manifest requires\ncascading updates, while the moon approach is ",(0,s.jsx)(n.code,{children:"O(1)"}),"!"]}),"\n",(0,s.jsxs)(a.Z,{groupId:"dockerfile",defaultValue:"standard",values:[{label:"Standard",value:"standard"},{label:"Multi-staged",value:"staged"}],children:[(0,s.jsx)(i.Z,{value:"standard",children:(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-docker",children:"FROM node:latest\nWORKDIR /app\n\n# Install moon binary\nRUN npm install -g @moonrepo/cli\n\n# Copy workspace skeleton\nCOPY ./.moon/docker/workspace .\n\n# Install toolchain and dependencies\nRUN moon docker setup\n\n# Copy source files\nCOPY ./.moon/docker/sources .\n\n# Run something\nRUN moon run <project>:<task>\n\n# Prune workspace\nRUN moon docker prune\n\n# Or CMD\n"})})}),(0,s.jsx)(i.Z,{value:"staged",children:(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-docker",children:"#### BASE\nFROM node:latest AS base\nWORKDIR /app\n\n# Install moon binary\nRUN npm install -g @moonrepo/cli\n\n#### WORKSPACE\nFROM base AS workspace\nWORKDIR /app\n\n# Copy entire repository and scaffold\nCOPY . .\nRUN moon docker scaffold <project>\n\n#### BUILD\nFROM base AS build\nWORKDIR /app\n\n# Copy workspace skeleton\nCOPY --from=workspace /app/.moon/docker/workspace .\n\n# Install toolchain and dependencies\nRUN moon docker setup\n\n# Copy source files\nCOPY --from=workspace /app/.moon/docker/sources .\n\n# Run something\nRUN moon run <project>:<task>\n\n# Prune workspace\nRUN moon docker prune\n\n# Or CMD\n"})})})]}),"\n",(0,s.jsx)(n.h2,{id:"troubleshooting",children:"Troubleshooting"}),"\n",(0,s.jsxs)(n.h3,{id:"supporting-nodealpine-images",children:["Supporting ",(0,s.jsx)(n.code,{children:"node:alpine"})," images"]}),"\n",(0,s.jsxs)(n.p,{children:["If you're trying to use the ",(0,s.jsx)(n.code,{children:"node:alpine"})," image with moon's\n",(0,s.jsx)(n.a,{href:"../concepts/toolchain",children:"integrated toolchain"}),", you'll need to set the ",(0,s.jsx)(n.code,{children:"MOON_TOOLCHAIN_FORCE_GLOBALS"}),"\nenvironment variable in the Docker image to disable moon's toolchain. This is required as Node.js\ndoes not provide pre-built binaries for the Alpine target, so installing the Node.js toolchain will\nfail."]}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-docker",children:"FROM node:alpine\n\nENV MOON_TOOLCHAIN_FORCE_GLOBALS=1\n"})})]})}function p(e={}){const{wrapper:n}={...(0,r.a)(),...e.components};return n?(0,s.jsx)(n,{...e,children:(0,s.jsx)(h,{...e})}):h(e)}},39798:(e,n,o)=>{o.d(n,{Z:()=>i});o(27378);var s=o(40624);const r={tabItem:"tabItem_wHwb"};var a=o(24246);function i(e){let{children:n,hidden:o,className:i}=e;return(0,a.jsx)("div",{role:"tabpanel",className:(0,s.Z)(r.tabItem,i),hidden:o,children:n})}},33337:(e,n,o)=>{o.d(n,{Z:()=>p});var s=o(27378),r=o(40624),a=o(83457),i=o(35595),t=o(76457);const l={tabList:"tabList_J5MA",tabItem:"tabItem_l0OV"};var c=o(24246);function d(e){let{className:n,block:o,selectedValue:s,selectValue:i,tabValues:t}=e;const d=[],{blockElementScrollPositionUntilNextRender:u}=(0,a.o5)(),h=e=>{const n=e.currentTarget,o=d.indexOf(n),r=t[o].value;r!==s&&(u(n),i(r))},p=e=>{let n=null;switch(e.key){case"Enter":h(e);break;case"ArrowRight":{const o=d.indexOf(e.currentTarget)+1;n=d[o]??d[0];break}case"ArrowLeft":{const o=d.indexOf(e.currentTarget)-1;n=d[o]??d[d.length-1];break}}n?.focus()};return(0,c.jsx)("ul",{role:"tablist","aria-orientation":"horizontal",className:(0,r.Z)("tabs",{"tabs--block":o},n),children:t.map((e=>{let{value:n,label:o,attributes:a}=e;return(0,c.jsx)("li",{role:"tab",tabIndex:s===n?0:-1,"aria-selected":s===n,ref:e=>d.push(e),onKeyDown:p,onClick:h,...a,className:(0,r.Z)("tabs__item",l.tabItem,a?.className,{"tabs__item--active":s===n}),children:o??n},n)}))})}function u(e){let{lazy:n,children:o,selectedValue:r}=e;const a=(Array.isArray(o)?o:[o]).filter(Boolean);if(n){const e=a.find((e=>e.props.value===r));return e?(0,s.cloneElement)(e,{className:"margin-top--md"}):null}return(0,c.jsx)("div",{className:"margin-top--md",children:a.map(((e,n)=>(0,s.cloneElement)(e,{key:n,hidden:e.props.value!==r})))})}function h(e){const n=(0,i.Y)(e);return(0,c.jsxs)("div",{className:(0,r.Z)("tabs-container",l.tabList),children:[(0,c.jsx)(d,{...n,...e}),(0,c.jsx)(u,{...n,...e})]})}function p(e){const n=(0,t.Z)();return(0,c.jsx)(h,{...e,children:(0,i.h)(e.children)},String(n))}},35595:(e,n,o)=>{o.d(n,{Y:()=>p,h:()=>c});var s=o(27378),r=o(3620),a=o(9834),i=o(30654),t=o(70784),l=o(71819);function c(e){return s.Children.toArray(e).filter((e=>"\n"!==e)).map((e=>{if(!e||(0,s.isValidElement)(e)&&function(e){const{props:n}=e;return!!n&&"object"==typeof n&&"value"in n}(e))return e;throw new Error(`Docusaurus error: Bad <Tabs> child <${"string"==typeof e.type?e.type:e.type.name}>: all children of the <Tabs> component should be <TabItem>, and every <TabItem> should have a unique "value" prop.`)}))?.filter(Boolean)??[]}function d(e){const{values:n,children:o}=e;return(0,s.useMemo)((()=>{const e=n??function(e){return c(e).map((e=>{let{props:{value:n,label:o,attributes:s,default:r}}=e;return{value:n,label:o,attributes:s,default:r}}))}(o);return function(e){const n=(0,t.l)(e,((e,n)=>e.value===n.value));if(n.length>0)throw new Error(`Docusaurus error: Duplicate values "${n.map((e=>e.value)).join(", ")}" found in <Tabs>. Every value needs to be unique.`)}(e),e}),[n,o])}function u(e){let{value:n,tabValues:o}=e;return o.some((e=>e.value===n))}function h(e){let{queryString:n=!1,groupId:o}=e;const a=(0,r.k6)(),t=function(e){let{queryString:n=!1,groupId:o}=e;if("string"==typeof n)return n;if(!1===n)return null;if(!0===n&&!o)throw new Error('Docusaurus error: The <Tabs> component groupId prop is required if queryString=true, because this value is used as the search param name. You can also provide an explicit value such as queryString="my-search-param".');return o??null}({queryString:n,groupId:o});return[(0,i._X)(t),(0,s.useCallback)((e=>{if(!t)return;const n=new URLSearchParams(a.location.search);n.set(t,e),a.replace({...a.location,search:n.toString()})}),[t,a])]}function p(e){const{defaultValue:n,queryString:o=!1,groupId:r}=e,i=d(e),[t,c]=(0,s.useState)((()=>function(e){let{defaultValue:n,tabValues:o}=e;if(0===o.length)throw new Error("Docusaurus error: the <Tabs> component requires at least one <TabItem> children component");if(n){if(!u({value:n,tabValues:o}))throw new Error(`Docusaurus error: The <Tabs> has a defaultValue "${n}" but none of its children has the corresponding value. Available values are: ${o.map((e=>e.value)).join(", ")}. If you intend to show no default tab, use defaultValue={null} instead.`);return n}const s=o.find((e=>e.default))??o[0];if(!s)throw new Error("Unexpected error: 0 tabValues");return s.value}({defaultValue:n,tabValues:i}))),[p,m]=h({queryString:o,groupId:r}),[f,g]=function(e){let{groupId:n}=e;const o=function(e){return e?`docusaurus.tab.${e}`:null}(n),[r,a]=(0,l.Nk)(o);return[r,(0,s.useCallback)((e=>{o&&a.set(e)}),[o,a])]}({groupId:r}),k=(()=>{const e=p??f;return u({value:e,tabValues:i})?e:null})();(0,a.Z)((()=>{k&&c(k)}),[k]);return{selectedValue:t,selectValue:(0,s.useCallback)((e=>{if(!u({value:e,tabValues:i}))throw new Error(`Can't select invalid tab value=${e}`);c(e),m(e),g(e)}),[m,g,i]),tabValues:i}}},71670:(e,n,o)=>{o.d(n,{Z:()=>t,a:()=>i});var s=o(27378);const r={},a=s.createContext(r);function i(e){const n=s.useContext(a);return s.useMemo((function(){return"function"==typeof e?e(n):{...n,...e}}),[n,e])}function t(e){let n;return n=e.disableParentContext?"function"==typeof e.components?e.components(r):e.components||r:i(e.components),s.createElement(a.Provider,{value:n},e.children)}}}]);