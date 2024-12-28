"use strict";(self.webpackChunkwebsite=self.webpackChunkwebsite||[]).push([[77363],{14909:(e,n,t)=>{t.r(n),t.d(n,{assets:()=>u,contentTitle:()=>i,default:()=>h,frontMatter:()=>a,metadata:()=>c,toc:()=>d});var o=t(24246),r=t(71670),s=t(33337),l=t(39798);const a={title:"completions"},i=void 0,c={id:"commands/completions",title:"completions",description:"The moon completions command will generate moon command and argument completions for your current",source:"@site/docs/commands/completions.mdx",sourceDirName:"commands",slug:"/commands/completions",permalink:"/docs/commands/completions",draft:!1,unlisted:!1,editUrl:"https://github.com/moonrepo/moon/tree/master/website/docs/commands/completions.mdx",tags:[],version:"current",frontMatter:{title:"completions"},sidebar:"docs",previous:{title:"clean",permalink:"/docs/commands/clean"},next:{title:"docker",permalink:"/docs/commands/docker"}},u={},d=[{value:"Options",id:"options",level:3},{value:"Examples",id:"examples",level:3}];function m(e){const n={a:"a",code:"code",h3:"h3",li:"li",p:"p",pre:"pre",ul:"ul",...(0,r.a)(),...e.components};return(0,o.jsxs)(o.Fragment,{children:[(0,o.jsxs)(n.p,{children:["The ",(0,o.jsx)(n.code,{children:"moon completions"})," command will generate moon command and argument completions for your current\nshell. This command will write to stdout, which can then be redirected to a file of your choice."]}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{className:"language-shell",children:"$ moon completions > ./path/to/write/to\n"})}),"\n",(0,o.jsx)(n.h3,{id:"options",children:"Options"}),"\n",(0,o.jsxs)(n.ul,{children:["\n",(0,o.jsxs)(n.li,{children:[(0,o.jsx)(n.code,{children:"--shell"})," - Shell to explicitly generate for."]}),"\n"]}),"\n",(0,o.jsx)(n.h3,{id:"examples",children:"Examples"}),"\n",(0,o.jsxs)(s.Z,{groupId:"comp",defaultValue:"bash",values:[{label:"Bash",value:"bash"},{label:"Fish",value:"fish"},{label:"Zsh",value:"zsh"}],children:[(0,o.jsxs)(l.Z,{value:"bash",children:[(0,o.jsxs)(n.p,{children:["If using ",(0,o.jsx)(n.a,{href:"https://github.com/scop/bash-completion",children:"bash-completion"}),"."]}),(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{className:"language-shell",children:"mkdir -p ~/.bash_completion.d\nmoon completions > ~/.bash_completion.d/moon.sh\n"})}),(0,o.jsx)(n.p,{children:"Otherwise write the file to a common location, and source it in your profile."}),(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{className:"language-shell",children:"mkdir -p ~/.bash_completions\nmoon completions > ~/.bash_completions/moon.sh\n\n# In your profile\nsource ~/.bash_completions/moon.sh\n"})})]}),(0,o.jsxs)(l.Z,{value:"fish",children:[(0,o.jsx)(n.p,{children:"Write the file to Fish's completions directory."}),(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{className:"language-shell",children:"mkdir -p ~/.config/fish/completions\nmoon completions > ~/.config/fish/completions/moon.fish\n"})})]}),(0,o.jsxs)(l.Z,{value:"zsh",children:[(0,o.jsxs)(n.p,{children:["If using ",(0,o.jsx)(n.a,{href:"https://ohmyz.sh/",children:"oh-my-zsh"})," (the ",(0,o.jsx)(n.code,{children:"_"})," prefix is required)."]}),(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{className:"language-shell",children:"mkdir -p ~/.oh-my-zsh/completions\nmoon completions > ~/.oh-my-zsh/completions/_moon\n\n# Reload shell (or restart terminal)\nomz reload\n"})})]})]})]})}function h(e={}){const{wrapper:n}={...(0,r.a)(),...e.components};return n?(0,o.jsx)(n,{...e,children:(0,o.jsx)(m,{...e})}):m(e)}},39798:(e,n,t)=>{t.d(n,{Z:()=>l});t(27378);var o=t(40624);const r={tabItem:"tabItem_wHwb"};var s=t(24246);function l(e){let{children:n,hidden:t,className:l}=e;return(0,s.jsx)("div",{role:"tabpanel",className:(0,o.Z)(r.tabItem,l),hidden:t,children:n})}},33337:(e,n,t)=>{t.d(n,{Z:()=>h});var o=t(27378),r=t(40624),s=t(83457),l=t(35595),a=t(76457);const i={tabList:"tabList_J5MA",tabItem:"tabItem_l0OV"};var c=t(24246);function u(e){let{className:n,block:t,selectedValue:o,selectValue:l,tabValues:a}=e;const u=[],{blockElementScrollPositionUntilNextRender:d}=(0,s.o5)(),m=e=>{const n=e.currentTarget,t=u.indexOf(n),r=a[t].value;r!==o&&(d(n),l(r))},h=e=>{let n=null;switch(e.key){case"Enter":m(e);break;case"ArrowRight":{const t=u.indexOf(e.currentTarget)+1;n=u[t]??u[0];break}case"ArrowLeft":{const t=u.indexOf(e.currentTarget)-1;n=u[t]??u[u.length-1];break}}n?.focus()};return(0,c.jsx)("ul",{role:"tablist","aria-orientation":"horizontal",className:(0,r.Z)("tabs",{"tabs--block":t},n),children:a.map((e=>{let{value:n,label:t,attributes:s}=e;return(0,c.jsx)("li",{role:"tab",tabIndex:o===n?0:-1,"aria-selected":o===n,ref:e=>u.push(e),onKeyDown:h,onClick:m,...s,className:(0,r.Z)("tabs__item",i.tabItem,s?.className,{"tabs__item--active":o===n}),children:t??n},n)}))})}function d(e){let{lazy:n,children:t,selectedValue:r}=e;const s=(Array.isArray(t)?t:[t]).filter(Boolean);if(n){const e=s.find((e=>e.props.value===r));return e?(0,o.cloneElement)(e,{className:"margin-top--md"}):null}return(0,c.jsx)("div",{className:"margin-top--md",children:s.map(((e,n)=>(0,o.cloneElement)(e,{key:n,hidden:e.props.value!==r})))})}function m(e){const n=(0,l.Y)(e);return(0,c.jsxs)("div",{className:(0,r.Z)("tabs-container",i.tabList),children:[(0,c.jsx)(u,{...n,...e}),(0,c.jsx)(d,{...n,...e})]})}function h(e){const n=(0,a.Z)();return(0,c.jsx)(m,{...e,children:(0,l.h)(e.children)},String(n))}},35595:(e,n,t)=>{t.d(n,{Y:()=>h,h:()=>c});var o=t(27378),r=t(3620),s=t(9834),l=t(30654),a=t(70784),i=t(55643);function c(e){return o.Children.toArray(e).filter((e=>"\n"!==e)).map((e=>{if(!e||(0,o.isValidElement)(e)&&function(e){const{props:n}=e;return!!n&&"object"==typeof n&&"value"in n}(e))return e;throw new Error(`Docusaurus error: Bad <Tabs> child <${"string"==typeof e.type?e.type:e.type.name}>: all children of the <Tabs> component should be <TabItem>, and every <TabItem> should have a unique "value" prop.`)}))?.filter(Boolean)??[]}function u(e){const{values:n,children:t}=e;return(0,o.useMemo)((()=>{const e=n??function(e){return c(e).map((e=>{let{props:{value:n,label:t,attributes:o,default:r}}=e;return{value:n,label:t,attributes:o,default:r}}))}(t);return function(e){const n=(0,a.l)(e,((e,n)=>e.value===n.value));if(n.length>0)throw new Error(`Docusaurus error: Duplicate values "${n.map((e=>e.value)).join(", ")}" found in <Tabs>. Every value needs to be unique.`)}(e),e}),[n,t])}function d(e){let{value:n,tabValues:t}=e;return t.some((e=>e.value===n))}function m(e){let{queryString:n=!1,groupId:t}=e;const s=(0,r.k6)(),a=function(e){let{queryString:n=!1,groupId:t}=e;if("string"==typeof n)return n;if(!1===n)return null;if(!0===n&&!t)throw new Error('Docusaurus error: The <Tabs> component groupId prop is required if queryString=true, because this value is used as the search param name. You can also provide an explicit value such as queryString="my-search-param".');return t??null}({queryString:n,groupId:t});return[(0,l._X)(a),(0,o.useCallback)((e=>{if(!a)return;const n=new URLSearchParams(s.location.search);n.set(a,e),s.replace({...s.location,search:n.toString()})}),[a,s])]}function h(e){const{defaultValue:n,queryString:t=!1,groupId:r}=e,l=u(e),[a,c]=(0,o.useState)((()=>function(e){let{defaultValue:n,tabValues:t}=e;if(0===t.length)throw new Error("Docusaurus error: the <Tabs> component requires at least one <TabItem> children component");if(n){if(!d({value:n,tabValues:t}))throw new Error(`Docusaurus error: The <Tabs> has a defaultValue "${n}" but none of its children has the corresponding value. Available values are: ${t.map((e=>e.value)).join(", ")}. If you intend to show no default tab, use defaultValue={null} instead.`);return n}const o=t.find((e=>e.default))??t[0];if(!o)throw new Error("Unexpected error: 0 tabValues");return o.value}({defaultValue:n,tabValues:l}))),[h,p]=m({queryString:t,groupId:r}),[f,b]=function(e){let{groupId:n}=e;const t=function(e){return e?`docusaurus.tab.${e}`:null}(n),[r,s]=(0,i.Nk)(t);return[r,(0,o.useCallback)((e=>{t&&s.set(e)}),[t,s])]}({groupId:r}),v=(()=>{const e=h??f;return d({value:e,tabValues:l})?e:null})();(0,s.Z)((()=>{v&&c(v)}),[v]);return{selectedValue:a,selectValue:(0,o.useCallback)((e=>{if(!d({value:e,tabValues:l}))throw new Error(`Can't select invalid tab value=${e}`);c(e),p(e),b(e)}),[p,b,l]),tabValues:l}}},71670:(e,n,t)=>{t.d(n,{Z:()=>a,a:()=>l});var o=t(27378);const r={},s=o.createContext(r);function l(e){const n=o.useContext(s);return o.useMemo((function(){return"function"==typeof e?e(n):{...n,...e}}),[n,e])}function a(e){let n;return n=e.disableParentContext?"function"==typeof e.components?e.components(r):e.components||r:l(e.components),o.createElement(s.Provider,{value:n},e.children)}}}]);