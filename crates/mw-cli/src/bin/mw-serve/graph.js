const cv=document.getElementById('g'),cx=cv.getContext('2d');
const W=cv.width,H=cv.height,N=DATA.nodes,L=DATA.links;
if(!N.length){cx.fillStyle='#566273';cx.font='15px sans-serif';cx.fillText('No commands with arguments yet — record some with mw-remember.',24,40);}
else{
const idx={},maxW=Math.max(1,...N.map(n=>n.weight||1));
N.forEach(n=>{idx[n.id]=n;n.x=W/2+(Math.random()-.5)*260;n.y=H/2+(Math.random()-.5)*260;n.vx=0;n.vy=0;n.r=(n.kind==='cmd'?8:4)+14*Math.sqrt((n.weight||1)/maxW);});
L.forEach(l=>{l.s=idx[l.source];l.t=idx[l.target];});
function col(n){return n.kind==='cmd'?'#2b43dd':n.kind==='bridge'?'#e9663a':'#10b6c6';}
function step(){
 for(let i=0;i<N.length;i++)for(let j=i+1;j<N.length;j++){const a=N[i],b=N[j];let dx=a.x-b.x,dy=a.y-b.y,d=Math.hypot(dx,dy)||1;if(d<320){const f=2600/(d*d);a.vx+=dx/d*f;a.vy+=dy/d*f;b.vx-=dx/d*f;b.vy-=dy/d*f;}}
 L.forEach(l=>{if(!l.s||!l.t)return;let dx=l.t.x-l.s.x,dy=l.t.y-l.s.y,d=Math.hypot(dx,dy)||1,f=(d-84)*0.02;l.s.vx+=dx/d*f;l.s.vy+=dy/d*f;l.t.vx-=dx/d*f;l.t.vy-=dy/d*f;});
 N.forEach(n=>{n.vx+=(W/2-n.x)*0.002;n.vy+=(H/2-n.y)*0.002;n.vx*=0.86;n.vy*=0.86;n.x+=n.vx;n.y+=n.vy;n.x=Math.max(30,Math.min(W-30,n.x));n.y=Math.max(30,Math.min(H-30,n.y));});
}
function draw(){
 cx.clearRect(0,0,W,H);
 cx.strokeStyle='#d5dee9';cx.lineWidth=1;
 L.forEach(l=>{if(!l.s||!l.t)return;cx.beginPath();cx.moveTo(l.s.x,l.s.y);cx.lineTo(l.t.x,l.t.y);cx.stroke();});
 N.forEach(n=>{cx.beginPath();cx.arc(n.x,n.y,n.r,0,7);cx.fillStyle=col(n);cx.fill();cx.fillStyle='#0f1722';cx.font=(n.kind==='cmd'?'600 12px ':'11px ')+'ui-monospace,monospace';cx.fillText(n.label,n.x+n.r+4,n.y+4);});
}
let t=0;function loop(){for(let k=0;k<3;k++)step();draw();if(t++<800)requestAnimationFrame(loop);}
loop();
cv.style.cursor='pointer';
cv.onclick=e=>{const rc=cv.getBoundingClientRect(),sx=W/rc.width,sy=H/rc.height,mx=(e.clientX-rc.left)*sx,my=(e.clientY-rc.top)*sy;let best=null,bd=1e9;N.forEach(n=>{const d=(n.x-mx)**2+(n.y-my)**2;if(d<bd&&d<(n.r+12)*(n.r+12)){bd=d;best=n;}});if(best&&best.kind==='cmd'&&best.name)location.href='/runs/'+encodeURIComponent(best.name);};
}
