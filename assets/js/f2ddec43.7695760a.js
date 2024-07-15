"use strict";(self.webpackChunkwebsite=self.webpackChunkwebsite||[]).push([[7574],{32267:(e,n,o)=>{o.r(n),o.d(n,{assets:()=>l,contentTitle:()=>r,default:()=>h,frontMatter:()=>i,metadata:()=>a,toc:()=>c});var t=o(24246),s=o(71670);const i={slug:"moon-v1.27",title:"moon v1.27 - Task scripts, Docker settings, and more",authors:["milesj"],tags:["action","pipeline","task","script","command","docker","dockerfile"],image:"./img/moon/v1.27.png"},r=void 0,a={permalink:"/blog/moon-v1.27",editUrl:"https://github.com/moonrepo/moon/tree/master/website/blog/2024-07-14_moon-v1.27.mdx",source:"@site/blog/2024-07-14_moon-v1.27.mdx",title:"moon v1.27 - Task scripts, Docker settings, and more",description:"In this release, we're adding improved Docker support, and a long-awaited task request.",date:"2024-07-14T00:00:00.000Z",tags:[{inline:!0,label:"action",permalink:"/blog/tags/action"},{inline:!0,label:"pipeline",permalink:"/blog/tags/pipeline"},{inline:!0,label:"task",permalink:"/blog/tags/task"},{inline:!0,label:"script",permalink:"/blog/tags/script"},{inline:!0,label:"command",permalink:"/blog/tags/command"},{inline:!0,label:"docker",permalink:"/blog/tags/docker"},{inline:!0,label:"dockerfile",permalink:"/blog/tags/dockerfile"}],readingTime:3.985,hasTruncateMarker:!0,authors:[{name:"Miles Johnson",title:"Founder, developer",url:"https://github.com/milesj",imageURL:"/img/authors/miles.jpg",key:"milesj"}],frontMatter:{slug:"moon-v1.27",title:"moon v1.27 - Task scripts, Docker settings, and more",authors:["milesj"],tags:["action","pipeline","task","script","command","docker","dockerfile"],image:"./img/moon/v1.27.png"},unlisted:!1,nextItem:{title:"proto v0.38 - Shell activation / hooks",permalink:"/blog/proto-v0.38"}},l={image:o(8873).Z,authorsImageUrls:[void 0]},c=[{value:"Experimental pipeline enabled by default",id:"experimental-pipeline-enabled-by-default",level:2},{value:"New task scripts",id:"new-task-scripts",level:2},{value:"Improved Docker integration",id:"improved-docker-integration",level:2},{value:"New <code>moon docker file</code> command",id:"new-moon-docker-file-command",level:3},{value:"New <code>docker</code> settings",id:"new-docker-settings",level:3},{value:"Other changes",id:"other-changes",level:2}];function d(e){const n={a:"a",admonition:"admonition",code:"code",h2:"h2",h3:"h3",li:"li",p:"p",pre:"pre",ul:"ul",...(0,s.a)(),...e.components};return(0,t.jsxs)(t.Fragment,{children:[(0,t.jsx)(n.p,{children:"In this release, we're adding improved Docker support, and a long-awaited task request."}),"\n",(0,t.jsx)(n.h2,{id:"experimental-pipeline-enabled-by-default",children:"Experimental pipeline enabled by default"}),"\n",(0,t.jsxs)(n.p,{children:["In our last release, we ",(0,t.jsx)(n.a,{href:"./moon-v1.26#new-experimental-pipeline",children:"introduced a new action pipeline"}),"\nthat is more performant, accurate, and resilient, but was hidden behind an experimental flag. Since\nthen, we've seen many users enable it successfully, and no issues have been reported! And with that,\nwe're enabling the experiment by default."]}),"\n",(0,t.jsxs)(n.p,{children:["If you run into an issue with this new pipeline, you can disable the experiment in\n",(0,t.jsx)(n.code,{children:".moon/workspace.yml"}),", like so. If you do encounter an issue, please report it to GitHub or Discord!"]}),"\n",(0,t.jsx)(n.pre,{children:(0,t.jsx)(n.code,{className:"language-yaml",metastring:'title=".moon/workspace.yml"',children:"experiments:\n  actionPipelineV2: false\n"})}),"\n",(0,t.jsx)(n.h2,{id:"new-task-scripts",children:"New task scripts"}),"\n",(0,t.jsx)(n.p,{children:"Since the beginning, tasks in moon have been modeled around a single command and its arguments; they\nare a 1-to-1 relationship. It was designed this way as it was a hard requirement for task\ninheritance to work correctly. If you have multiple tasks in the chain that need to be merged\ntogether, how will arguments be handled? Do they merge, overwrite, or replace? Do they prepend or\nappend? Or maybe you want to keep the arguments but rename the binary/command itself? And many more\nsuch combinations."}),"\n",(0,t.jsxs)(n.p,{children:["But because of this limitation, tasks did not support executing multiple commands (via ",(0,t.jsx)(n.code,{children:"&&"})," or ",(0,t.jsx)(n.code,{children:";"}),"),\nas this breaks argument merging (which command should the arguments belong too?). Tasks also did not\nsupport redirects, pipes, and other shell scripting syntax. Over the year we've slowly tried to\nsupport these in tasks, and while they technically do in some capacity, the experience is subpar."]}),"\n",(0,t.jsxs)(n.p,{children:["To remedy this, we're introducing a new task field called ",(0,t.jsx)(n.a,{href:"/docs/config/project#script",children:(0,t.jsx)(n.code,{children:"script"})}),",\nwhich is an alternative to ",(0,t.jsx)(n.a,{href:"/docs/config/project#command",children:(0,t.jsx)(n.code,{children:"command"})})," +\n",(0,t.jsx)(n.a,{href:"/docs/config/project#args",children:(0,t.jsx)(n.code,{children:"args"})}),". Scripts support everything mentioned above, and can be defined\nas such (using a very contrived example)."]}),"\n",(0,t.jsx)(n.pre,{children:(0,t.jsx)(n.code,{className:"language-yaml",metastring:'title="moon.yml"',children:"tasks:\n  build:\n    script: 'rm -rf ./out && ./doBuild.sh out && ./genStats.sh > stats.json'\n    outputs:\n      - 'out'\n"})}),"\n",(0,t.jsxs)(n.ul,{children:["\n",(0,t.jsx)(n.li,{children:"Scripts do support multiple commands, redirects, and pipes, while command/args do not."}),"\n",(0,t.jsx)(n.li,{children:"Scripts do not support argument task inheritance merging, while command/args do."}),"\n",(0,t.jsxs)(n.li,{children:["Scripts do not support passthrough arguments (after ",(0,t.jsx)(n.code,{children:"--"}),"), while command/args do."]}),"\n",(0,t.jsx)(n.li,{children:"Scripts can only be defined with a string, while command/args support a string or array."}),"\n",(0,t.jsx)(n.li,{children:"Both scripts and commands support the toolchain."}),"\n",(0,t.jsx)(n.li,{children:"Both scripts and commands support task tokens and variables."}),"\n"]}),"\n",(0,t.jsx)(n.admonition,{type:"info",children:(0,t.jsxs)(n.p,{children:["For a full list of comparisons, and more information on commands vs scripts, head over to the\n",(0,t.jsx)(n.a,{href:"/docs/concepts/task#commands-vs-scripts",children:"official task documentation"}),"!"]})}),"\n",(0,t.jsx)(n.h2,{id:"improved-docker-integration",children:"Improved Docker integration"}),"\n",(0,t.jsx)(n.p,{children:"As it turns out, a lot of moon users rely heavily on our Docker integration, which hasn't seen some\nlove in quite a while. We felt it was time to change that."}),"\n",(0,t.jsxs)(n.h3,{id:"new-moon-docker-file-command",children:["New ",(0,t.jsx)(n.code,{children:"moon docker file"})," command"]}),"\n",(0,t.jsxs)(n.p,{children:["Since our release of Docker in moon (v0.15), we've provided an ",(0,t.jsx)(n.a,{href:"/docs/guides/docker",children:"in-depth guide"}),"\non why our integration exists, and what it aims to solve. However, the guide required a bit of\nmanual non-trivial setup, which left users confused more than we like. To remedy this, we're\nintroducing a new command, ",(0,t.jsx)(n.a,{href:"/docs/commands/docker/file",children:(0,t.jsx)(n.code,{children:"moon docker file"})}),", which will generate a\nmulti-staged ",(0,t.jsx)(n.code,{children:"Dockerfile"})," for a project."]}),"\n",(0,t.jsxs)(n.p,{children:["To demonstrate this, here's what the ",(0,t.jsx)(n.code,{children:"Dockerfile"})," looks like for our website."]}),"\n",(0,t.jsx)(n.pre,{children:(0,t.jsx)(n.code,{className:"language-docker",children:'#### BASE STAGE\n#### Installs moon.\n\nFROM node:latest AS base\nWORKDIR /app\n\n# Install moon binary\nRUN curl -fsSL https://moonrepo.dev/install/moon.sh | bash\nENV PATH="/root/.moon/bin:$PATH"\n\n#### SKELETON STAGE\n#### Scaffolds repository skeleton structures.\n\nFROM base AS skeleton\n\n# Copy entire repository and scaffold\nCOPY . .\nRUN moon docker scaffold website\n\n#### BUILD STAGE\n#### Builds the project.\n\nFROM base AS build\n\n# Copy toolchain\nCOPY --from=skeleton /root/.proto /root/.proto\n\n# Copy workspace configs\nCOPY --from=skeleton /app/.moon/docker/workspace .\n\n# Install dependencies\nRUN moon docker setup\n\n# Copy project sources\nCOPY --from=skeleton /app/.moon/docker/sources .\n\n# Build the project\nRUN moon run website:build\n\n# Prune extraneous dependencies\nRUN moon docker prune\n\n#### START STAGE\n#### Runs the project.\n\nFROM base AS start\n\n# Copy built sources\nCOPY --from=build /root/.proto /root/.proto\nCOPY --from=build /app /app\n\nCMD moon run website:start\n'})}),"\n",(0,t.jsxs)(n.h3,{id:"new-docker-settings",children:["New ",(0,t.jsx)(n.code,{children:"docker"})," settings"]}),"\n",(0,t.jsxs)(n.p,{children:["To further improve our Docker support, we're also introducing new ",(0,t.jsx)(n.code,{children:"docker"})," settings to both\n",(0,t.jsx)(n.a,{href:"/docs/config/workspace#docker",children:(0,t.jsx)(n.code,{children:".moon/workspace.yml"})})," and\n",(0,t.jsx)(n.a,{href:"/docs/config/project#docker",children:(0,t.jsx)(n.code,{children:"moon.yml"})}),". These settings allow you to customize the scaffold,\nprune, and ",(0,t.jsx)(n.code,{children:"Dockerfile"})," generation flows."]}),"\n",(0,t.jsx)(n.pre,{children:(0,t.jsx)(n.code,{className:"language-yaml",metastring:'title=".moon/workspace.yml"',children:"docker:\n  prune:\n    installToolchainDeps: false\n  scaffold:\n    include:\n      - '*.config.js'\n"})}),"\n",(0,t.jsx)(n.pre,{children:(0,t.jsx)(n.code,{className:"language-yaml",metastring:'title="moon.yml"',children:"docker:\n  file:\n    image: 'node:latest'\n    buildTask: 'build'\n    startTask: 'start'\n"})}),"\n",(0,t.jsx)(n.h2,{id:"other-changes",children:"Other changes"}),"\n",(0,t.jsxs)(n.p,{children:["View the ",(0,t.jsx)(n.a,{href:"https://github.com/moonrepo/moon/releases/tag/v1.27.0",children:"official release"})," for a full list\nof changes."]}),"\n",(0,t.jsxs)(n.ul,{children:["\n",(0,t.jsxs)(n.li,{children:["Added support for ",(0,t.jsx)(n.a,{href:"https://murex.rocks/",children:"murex"})," shells."]}),"\n",(0,t.jsx)(n.li,{children:'Improved the "automatically install dependencies if a manifest/lockfile has changed" flow. This\nshould trigger less than before.'}),"\n",(0,t.jsxs)(n.li,{children:["We now generate JSON schemas for our configuration files to ",(0,t.jsx)(n.code,{children:".moon/cache/schemas"}),", so that they\ncan be dynamically created based on the current moon version and environment."]}),"\n",(0,t.jsx)(n.li,{children:"When writing JSON and YAML files, we attempt to write back to the file with its original\nindentation."}),"\n"]})]})}function h(e={}){const{wrapper:n}={...(0,s.a)(),...e.components};return n?(0,t.jsx)(n,{...e,children:(0,t.jsx)(d,{...e})}):d(e)}},8873:(e,n,o)=>{o.d(n,{Z:()=>t});const t=o.p+"assets/images/v1.27-2eea05e2a100b547237b9e7166784ab6.png"},71670:(e,n,o)=>{o.d(n,{Z:()=>a,a:()=>r});var t=o(27378);const s={},i=t.createContext(s);function r(e){const n=t.useContext(i);return t.useMemo((function(){return"function"==typeof e?e(n):{...n,...e}}),[n,e])}function a(e){let n;return n=e.disableParentContext?"function"==typeof e.components?e.components(s):e.components||s:r(e.components),t.createElement(i.Provider,{value:n},e.children)}}}]);