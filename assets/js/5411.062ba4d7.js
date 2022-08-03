"use strict";(self.webpackChunkwebsite=self.webpackChunkwebsite||[]).push([[5411],{3635:function(e,n,t){t.d(n,{Z:function(){return m}});var r=t(5773),a=t(808),i=t(7378),l=t(8944),c=t(9213),u=t(624),o="anchorWithStickyNavbar_YDjN",s="anchorWithHideOnScrollNavbar_c5FC",d=["as","id"];function m(e){var n=e.as,t=e.id,m=(0,a.Z)(e,d),f=(0,u.L)().navbar.hideOnScroll;return"h1"!==n&&t?i.createElement(n,(0,r.Z)({},m,{className:(0,l.Z)("anchor",f?s:o),id:t}),m.children,i.createElement("a",{className:"hash-link",href:"#"+t,title:(0,c.I)({id:"theme.common.headingLinkTitle",message:"Direct link to heading",description:"Title for link to heading"})},"\u200b")):i.createElement(n,(0,r.Z)({},m,{id:void 0}))}},1788:function(e,n,t){t.r(n),t.d(n,{default:function(){return H}});var r=t(5773),a=t(7378),i=t(808),l=t(7092),c=["mdxType","originalType"];var u=t(9589);var o=t(1884);var s=t(8944),d=t(6457),m=t(376),f="details_IpIu",v="isBrowser_QD4r",p="collapsibleContent_Fd2D",h=["summary","children"];function g(e){return!!e&&("SUMMARY"===e.tagName||g(e.parentElement))}function E(e,n){return!!e&&(e===n||E(e.parentElement,n))}function y(e){var n=e.summary,t=e.children,r=(0,i.Z)(e,h),l=(0,d.Z)(),c=(0,a.useRef)(null),u=(0,m.u)({initialState:!r.open}),o=u.collapsed,y=u.setCollapsed,Z=(0,a.useState)(r.open),L=Z[0],b=Z[1];return a.createElement("details",Object.assign({},r,{ref:c,open:L,"data-collapsed":o,className:(0,s.Z)(f,l&&v,r.className),onMouseDown:function(e){g(e.target)&&e.detail>1&&e.preventDefault()},onClick:function(e){e.stopPropagation();var n=e.target;g(n)&&E(n,c.current)&&(e.preventDefault(),o?(y(!1),b(!0)):y(!0))}}),n||a.createElement("summary",null,"Details"),a.createElement(m.z,{lazy:!1,collapsed:o,disableSSRStyle:!0,onCollapseTransitionEnd:function(e){y(e),b(!e)}},a.createElement("div",{className:p},t)))}var Z="details_TBmf";function L(e){var n=Object.assign({},e);return a.createElement(y,(0,r.Z)({},n,{className:(0,s.Z)("alert alert--info",Z,n.className)}))}var b=t(3635);function x(e){return a.createElement(b.Z,e)}var N="img_PFMr";var H={head:function(e){var n=a.Children.map(e.children,(function(e){return function(e){var n,t;if(null!=e&&null!=(n=e.props)&&n.mdxType&&null!=e&&null!=(t=e.props)&&t.originalType){var r=e.props,l=(r.mdxType,r.originalType,(0,i.Z)(r,c));return a.createElement(e.props.originalType,l)}return e}(e)}));return a.createElement(l.Z,e,n)},code:function(e){var n=["a","b","big","i","span","em","strong","sup","sub","small"];return a.Children.toArray(e.children).every((function(e){return"string"==typeof e&&!e.includes("\n")||(0,a.isValidElement)(e)&&n.includes(e.props.mdxType)}))?a.createElement("code",e):a.createElement(u.Z,e)},a:function(e){return a.createElement(o.Z,e)},pre:function(e){var n;return a.createElement(u.Z,(0,a.isValidElement)(e.children)&&"code"===e.children.props.originalType?null==(n=e.children)?void 0:n.props:Object.assign({},e))},details:function(e){var n=a.Children.toArray(e.children),t=n.find((function(e){var n;return"summary"===(null==e||null==(n=e.props)?void 0:n.mdxType)})),i=a.createElement(a.Fragment,null,n.filter((function(e){return e!==t})));return a.createElement(L,(0,r.Z)({},e,{summary:t}),i)},ul:function(e){return a.createElement("ul",(0,r.Z)({},e,{className:(n=e.className,(0,s.Z)(n,(null==n?void 0:n.includes("contains-task-list"))&&"clean-list"))}));var n},img:function(e){return a.createElement("img",(0,r.Z)({loading:"lazy"},e,{className:(n=e.className,(0,s.Z)(n,N))}));var n},h1:function(e){return a.createElement(x,(0,r.Z)({as:"h1"},e))},h2:function(e){return a.createElement(x,(0,r.Z)({as:"h2"},e))},h3:function(e){return a.createElement(x,(0,r.Z)({as:"h3"},e))},h4:function(e){return a.createElement(x,(0,r.Z)({as:"h4"},e))},h5:function(e){return a.createElement(x,(0,r.Z)({as:"h5"},e))},h6:function(e){return a.createElement(x,(0,r.Z)({as:"h6"},e))}}},1344:function(e,n,t){t.d(n,{S:function(){return u}});var r=t(7378),a=t(624);function i(e){var n=e.getBoundingClientRect();return n.top===n.bottom?i(e.parentNode):n}function l(e,n){var t,r,a=n.anchorTopOffset,l=e.find((function(e){return i(e).top>=a}));return l?function(e){return e.top>0&&e.bottom<window.innerHeight/2}(i(l))?l:null!=(r=e[e.indexOf(l)-1])?r:null:null!=(t=e[e.length-1])?t:null}function c(){var e=(0,r.useRef)(0),n=(0,a.L)().navbar.hideOnScroll;return(0,r.useEffect)((function(){e.current=n?0:document.querySelector(".navbar").clientHeight}),[n]),e}function u(e){var n=(0,r.useRef)(void 0),t=c();(0,r.useEffect)((function(){if(!e)return function(){};var r=e.linkClassName,a=e.linkActiveClassName,i=e.minHeadingLevel,c=e.maxHeadingLevel;function u(){var e=function(e){return Array.from(document.getElementsByClassName(e))}(r),u=function(e){for(var n=e.minHeadingLevel,t=e.maxHeadingLevel,r=[],a=n;a<=t;a+=1)r.push("h"+a+".anchor");return Array.from(document.querySelectorAll(r.join()))}({minHeadingLevel:i,maxHeadingLevel:c}),o=l(u,{anchorTopOffset:t.current}),s=e.find((function(e){return o&&o.id===function(e){return decodeURIComponent(e.href.substring(e.href.indexOf("#")+1))}(e)}));e.forEach((function(e){!function(e,t){if(t){var r;n.current&&n.current!==e&&(null==(r=n.current)||r.classList.remove(a)),e.classList.add(a),n.current=e}else e.classList.remove(a)}(e,e===s)}))}return document.addEventListener("scroll",u),document.addEventListener("resize",u),u(),function(){document.removeEventListener("scroll",u),document.removeEventListener("resize",u)}}),[e,t])}},6934:function(e,n,t){t.d(n,{a:function(){return c},b:function(){return o}});var r=t(808),a=t(7378),i=["parentIndex"];function l(e){var n=e.map((function(e){return Object.assign({},e,{parentIndex:-1,children:[]})})),t=Array(7).fill(-1);n.forEach((function(e,n){var r=t.slice(2,e.level);e.parentIndex=Math.max.apply(Math,r),t[e.level]=n}));var a=[];return n.forEach((function(e){var t=e.parentIndex,l=(0,r.Z)(e,i);t>=0?n[t].children.push(l):a.push(l)})),a}function c(e){return(0,a.useMemo)((function(){return l(e)}),[e])}function u(e){var n=e.toc,t=e.minHeadingLevel,r=e.maxHeadingLevel;return n.flatMap((function(e){var n=u({toc:e.children,minHeadingLevel:t,maxHeadingLevel:r});return function(e){return e.level>=t&&e.level<=r}(e)?[Object.assign({},e,{children:n})]:n}))}function o(e){var n=e.toc,t=e.minHeadingLevel,r=e.maxHeadingLevel;return(0,a.useMemo)((function(){return u({toc:l(n),minHeadingLevel:t,maxHeadingLevel:r})}),[n,t,r])}}}]);