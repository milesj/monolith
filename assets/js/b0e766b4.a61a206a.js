"use strict";(self.webpackChunkwebsite=self.webpackChunkwebsite||[]).push([[12949],{59776:(e,t,n)=>{n.r(t),n.d(t,{assets:()=>u,contentTitle:()=>l,default:()=>d,frontMatter:()=>o,metadata:()=>s,toc:()=>i});var r=n(24246),a=n(71670);n(33337),n(39798);const o={slug:"v0.15",title:"moon v0.15 - Enhanced Docker support and 1,000 stars!",authors:["milesj"],tags:["generator","docker"],image:"./img/v0.15.png"},l=void 0,s={permalink:"/blog/v0.15",editUrl:"https://github.com/moonrepo/moon/tree/master/website/blog/2022-09-26_v0.15.mdx",source:"@site/blog/2022-09-26_v0.15.mdx",title:"moon v0.15 - Enhanced Docker support and 1,000 stars!",description:"With this release, we've focused heavily on Docker integration and enhancing the Dockerfile",date:"2022-09-26T00:00:00.000Z",tags:[{inline:!0,label:"generator",permalink:"/blog/tags/generator"},{inline:!0,label:"docker",permalink:"/blog/tags/docker"}],readingTime:3.495,hasTruncateMarker:!0,authors:[{name:"Miles Johnson",title:"Founder, developer",url:"https://github.com/milesj",imageURL:"/img/authors/miles.jpg",key:"milesj"}],frontMatter:{slug:"v0.15",title:"moon v0.15 - Enhanced Docker support and 1,000 stars!",authors:["milesj"],tags:["generator","docker"],image:"./img/v0.15.png"},unlisted:!1,prevItem:{title:"moon v0.16 - Per-project tool versions and TypeScript improvements",permalink:"/blog/v0.16"},nextItem:{title:"moon v0.14 - Code generation and implicit dependencies",permalink:"/blog/v0.14"}},u={image:n(46496).Z,authorsImageUrls:[void 0]},i=[];function c(e){const t={code:"code",p:"p",...(0,a.a)(),...e.components};return(0,r.jsxs)(t.p,{children:["With this release, we've focused heavily on Docker integration and enhancing the ",(0,r.jsx)(t.code,{children:"Dockerfile"}),"\nworkflow, as well as some minor quality of life improvements for template files and run reports."]})}function d(e={}){const{wrapper:t}={...(0,a.a)(),...e.components};return t?(0,r.jsx)(t,{...e,children:(0,r.jsx)(c,{...e})}):c(e)}},39798:(e,t,n)=>{n.d(t,{Z:()=>l});n(27378);var r=n(40624);const a={tabItem:"tabItem_wHwb"};var o=n(24246);function l(e){let{children:t,hidden:n,className:l}=e;return(0,o.jsx)("div",{role:"tabpanel",className:(0,r.Z)(a.tabItem,l),hidden:n,children:t})}},33337:(e,t,n)=>{n.d(t,{Z:()=>p});var r=n(27378),a=n(40624),o=n(83457),l=n(35595),s=n(76457);const u={tabList:"tabList_J5MA",tabItem:"tabItem_l0OV"};var i=n(24246);function c(e){let{className:t,block:n,selectedValue:r,selectValue:l,tabValues:s}=e;const c=[],{blockElementScrollPositionUntilNextRender:d}=(0,o.o5)(),m=e=>{const t=e.currentTarget,n=c.indexOf(t),a=s[n].value;a!==r&&(d(t),l(a))},p=e=>{let t=null;switch(e.key){case"Enter":m(e);break;case"ArrowRight":{const n=c.indexOf(e.currentTarget)+1;t=c[n]??c[0];break}case"ArrowLeft":{const n=c.indexOf(e.currentTarget)-1;t=c[n]??c[c.length-1];break}}t?.focus()};return(0,i.jsx)("ul",{role:"tablist","aria-orientation":"horizontal",className:(0,a.Z)("tabs",{"tabs--block":n},t),children:s.map((e=>{let{value:t,label:n,attributes:o}=e;return(0,i.jsx)("li",{role:"tab",tabIndex:r===t?0:-1,"aria-selected":r===t,ref:e=>c.push(e),onKeyDown:p,onClick:m,...o,className:(0,a.Z)("tabs__item",u.tabItem,o?.className,{"tabs__item--active":r===t}),children:n??t},t)}))})}function d(e){let{lazy:t,children:n,selectedValue:a}=e;const o=(Array.isArray(n)?n:[n]).filter(Boolean);if(t){const e=o.find((e=>e.props.value===a));return e?(0,r.cloneElement)(e,{className:"margin-top--md"}):null}return(0,i.jsx)("div",{className:"margin-top--md",children:o.map(((e,t)=>(0,r.cloneElement)(e,{key:t,hidden:e.props.value!==a})))})}function m(e){const t=(0,l.Y)(e);return(0,i.jsxs)("div",{className:(0,a.Z)("tabs-container",u.tabList),children:[(0,i.jsx)(c,{...t,...e}),(0,i.jsx)(d,{...t,...e})]})}function p(e){const t=(0,s.Z)();return(0,i.jsx)(m,{...e,children:(0,l.h)(e.children)},String(t))}},35595:(e,t,n)=>{n.d(t,{Y:()=>p,h:()=>i});var r=n(27378),a=n(3620),o=n(9834),l=n(30654),s=n(70784),u=n(55643);function i(e){return r.Children.toArray(e).filter((e=>"\n"!==e)).map((e=>{if(!e||(0,r.isValidElement)(e)&&function(e){const{props:t}=e;return!!t&&"object"==typeof t&&"value"in t}(e))return e;throw new Error(`Docusaurus error: Bad <Tabs> child <${"string"==typeof e.type?e.type:e.type.name}>: all children of the <Tabs> component should be <TabItem>, and every <TabItem> should have a unique "value" prop.`)}))?.filter(Boolean)??[]}function c(e){const{values:t,children:n}=e;return(0,r.useMemo)((()=>{const e=t??function(e){return i(e).map((e=>{let{props:{value:t,label:n,attributes:r,default:a}}=e;return{value:t,label:n,attributes:r,default:a}}))}(n);return function(e){const t=(0,s.l)(e,((e,t)=>e.value===t.value));if(t.length>0)throw new Error(`Docusaurus error: Duplicate values "${t.map((e=>e.value)).join(", ")}" found in <Tabs>. Every value needs to be unique.`)}(e),e}),[t,n])}function d(e){let{value:t,tabValues:n}=e;return n.some((e=>e.value===t))}function m(e){let{queryString:t=!1,groupId:n}=e;const o=(0,a.k6)(),s=function(e){let{queryString:t=!1,groupId:n}=e;if("string"==typeof t)return t;if(!1===t)return null;if(!0===t&&!n)throw new Error('Docusaurus error: The <Tabs> component groupId prop is required if queryString=true, because this value is used as the search param name. You can also provide an explicit value such as queryString="my-search-param".');return n??null}({queryString:t,groupId:n});return[(0,l._X)(s),(0,r.useCallback)((e=>{if(!s)return;const t=new URLSearchParams(o.location.search);t.set(s,e),o.replace({...o.location,search:t.toString()})}),[s,o])]}function p(e){const{defaultValue:t,queryString:n=!1,groupId:a}=e,l=c(e),[s,i]=(0,r.useState)((()=>function(e){let{defaultValue:t,tabValues:n}=e;if(0===n.length)throw new Error("Docusaurus error: the <Tabs> component requires at least one <TabItem> children component");if(t){if(!d({value:t,tabValues:n}))throw new Error(`Docusaurus error: The <Tabs> has a defaultValue "${t}" but none of its children has the corresponding value. Available values are: ${n.map((e=>e.value)).join(", ")}. If you intend to show no default tab, use defaultValue={null} instead.`);return t}const r=n.find((e=>e.default))??n[0];if(!r)throw new Error("Unexpected error: 0 tabValues");return r.value}({defaultValue:t,tabValues:l}))),[p,h]=m({queryString:n,groupId:a}),[f,b]=function(e){let{groupId:t}=e;const n=function(e){return e?`docusaurus.tab.${e}`:null}(t),[a,o]=(0,u.Nk)(n);return[a,(0,r.useCallback)((e=>{n&&o.set(e)}),[n,o])]}({groupId:a}),v=(()=>{const e=p??f;return d({value:e,tabValues:l})?e:null})();(0,o.Z)((()=>{v&&i(v)}),[v]);return{selectedValue:s,selectValue:(0,r.useCallback)((e=>{if(!d({value:e,tabValues:l}))throw new Error(`Can't select invalid tab value=${e}`);i(e),h(e),b(e)}),[h,b,l]),tabValues:l}}},46496:(e,t,n)=>{n.d(t,{Z:()=>r});const r=n.p+"assets/images/v0.15-df3082f5cae38090b567718791719d91.png"},71670:(e,t,n)=>{n.d(t,{Z:()=>s,a:()=>l});var r=n(27378);const a={},o=r.createContext(a);function l(e){const t=r.useContext(o);return r.useMemo((function(){return"function"==typeof e?e(t):{...t,...e}}),[t,e])}function s(e){let t;return t=e.disableParentContext?"function"==typeof e.components?e.components(a):e.components||a:l(e.components),r.createElement(o.Provider,{value:t},e.children)}}}]);