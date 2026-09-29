// Hero ocean: a procedural low-poly whale, light rays, drifting particles,
// a fish school, and jellyfish in the deep. Decorative only (aria-hidden).
// Renders only while the hero is on screen, draws one still frame under
// prefers-reduced-motion, and leaves the CSS gradient in place without WebGL.
import * as THREE from "./vendor/three.module.min.js";

const canvas = document.querySelector(".ocean-canvas");
const hero = document.querySelector(".hero");
const reduce = matchMedia("(prefers-reduced-motion: reduce)").matches;

// Replay the recall transcript in the hero device: commands type out, output
// lines appear. The markup is the page's own static transcript, so without
// JavaScript (or with reduced motion) the full text simply stays visible.
const demo = document.querySelector(".recall-demo");
if (demo && !reduce) {
  const PROMPT = '<span class="prompt">$</span> ';
  const lines = demo.innerHTML.split("\n");
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  (async () => {
    for (;;) {
      let shown = "";
      for (const line of lines) {
        if (line.startsWith(PROMPT)) {
          const cmd = line.slice(PROMPT.length);
          for (let i = 1; i <= cmd.length; i++) {
            demo.innerHTML = `${shown}${PROMPT}${cmd.slice(0, i)}<span class="recall-caret"></span>`;
            await sleep(40);
          }
          await sleep(380);
        } else {
          await sleep(240);
        }
        shown += line + "\n";
        demo.innerHTML = shown;
      }
      demo.innerHTML = `${shown}${PROMPT}<span class="recall-caret"></span>`;
      await sleep(5200);
    }
  })();
}

let renderer;
try {
  renderer = canvas && new THREE.WebGLRenderer({ canvas, antialias: true });
} catch {
  renderer = null; // no WebGL: the gradient background stays
}

if (renderer) {
  renderer.setPixelRatio(Math.min(devicePixelRatio, 2));
  renderer.outputColorSpace = THREE.SRGBColorSpace;
  renderer.toneMapping = THREE.ACESFilmicToneMapping;
  hero.classList.add("has-ocean");
  start();
}

function start() {
  const scene = new THREE.Scene();
  scene.fog = new THREE.FogExp2(0x062437, 0.028);
  const camera = new THREE.PerspectiveCamera(45, 1, 0.1, 200);
  camera.position.set(0, 0, 22);

  // Backdrop: surface light above, deep water below, with a soft caustic shimmer.
  const bg = new THREE.Mesh(
    new THREE.SphereGeometry(100, 32, 16),
    new THREE.ShaderMaterial({
      side: THREE.BackSide, depthWrite: false, fog: false,
      uniforms: { top: { value: new THREE.Color("#1a7a8c") }, bot: { value: new THREE.Color("#03101c") }, time: { value: 0 } },
      vertexShader: "varying float h; varying vec3 vp; void main(){ vp = position; h = normalize(position).y; gl_Position = projectionMatrix*modelViewMatrix*vec4(position,1.); }",
      fragmentShader: "uniform vec3 top; uniform vec3 bot; uniform float time; varying float h; varying vec3 vp; void main(){ vec3 c = mix(bot, top, smoothstep(-0.25, 0.75, h)); float k = sin(vp.x*.18+time*.5)*sin(vp.z*.21-time*.4)+sin((vp.x+vp.z)*.11+time*.7); c += vec3(.35,.9,.85)*pow(max(k*.5,0.),3.)*smoothstep(.25,.8,h)*.35; gl_FragColor = vec4(c, 1.); }",
    })
  );
  scene.add(bg);

  scene.add(new THREE.HemisphereLight(0x9ff5ff, 0x02121e, 1.4));
  const sun = new THREE.DirectionalLight(0xdffcff, 2.2);
  sun.position.set(-4, 12, 6);
  const rim = new THREE.DirectionalLight(0x2dd4bf, 1.4);
  rim.position.set(6, -3, -6);
  scene.add(sun, rim);

  // ---- whale: a lathe body with fins, flukes, and vertex colours
  const L = 12;
  const profile = (t) => {
    const head = Math.sin(Math.min(t / 0.32, 1) * Math.PI / 2);
    const taper = t < 0.32 ? 1 : Math.pow(Math.cos((t - 0.32) / 0.68 * Math.PI / 2), 1.25);
    return 1.9 * head * taper + 0.12 * t;
  };
  const pts = [];
  for (let i = 0; i <= 40; i++) pts.push(new THREE.Vector2(Math.max(profile(i / 40), 0.001), (i / 40) * L));
  const lathe = new THREE.LatheGeometry(pts, 18);
  lathe.rotateZ(Math.PI / 2);
  lathe.translate(L * 0.5, 0, 0);
  {
    const p = lathe.attributes.position, colors = [];
    const back = new THREE.Color("#1f5f7a"), belly = new THREE.Color("#cfeee9");
    for (let i = 0; i < p.count; i++) {
      let y = p.getY(i);
      if (y < 0) y *= 0.78; // flatter belly
      p.setY(i, y);
      p.setZ(i, p.getZ(i) * 1.08);
      const c = back.clone().lerp(belly, THREE.MathUtils.smoothstep(-y, 0.15, 0.9));
      colors.push(c.r, c.g, c.b);
    }
    lathe.setAttribute("color", new THREE.Float32BufferAttribute(colors, 3));
  }
  const bodyGeo = lathe.toNonIndexed();
  bodyGeo.computeVertexNormals();
  const body = new THREE.Mesh(bodyGeo, new THREE.MeshStandardMaterial({ vertexColors: true, flatShading: true, roughness: 0.55, metalness: 0.05 }));
  const basePos = bodyGeo.attributes.position.array.slice();

  const finMat = new THREE.MeshStandardMaterial({ color: "#1c5670", flatShading: true, roughness: 0.6, side: THREE.DoubleSide });
  const shape = (list) => { const s = new THREE.Shape(); s.moveTo(...list[0]); list.slice(1).forEach((q) => s.lineTo(...q)); return s; };
  const extrude = { depth: 0.12, bevelEnabled: true, bevelThickness: 0.05, bevelSize: 0.05, bevelSegments: 1 };
  const flukeGeo = new THREE.ExtrudeGeometry(shape([[0, 0], [-0.6, 1.6], [-1.9, 2.7], [-1.4, 1.3], [-1.9, 0.15], [-1.9, -0.15], [-1.4, -1.3], [-1.9, -2.7], [-0.6, -1.6]]), extrude);
  flukeGeo.rotateX(Math.PI / 2);
  const fluke = new THREE.Mesh(flukeGeo, finMat);
  const finGeo = new THREE.ExtrudeGeometry(shape([[0, 0], [0.5, 0.1], [-0.9, -2.6], [-1.4, -2.4]]), extrude);
  const finL = new THREE.Mesh(finGeo, finMat), finR = new THREE.Mesh(finGeo, finMat);
  finL.position.set(2.6, -0.9, 1.3);
  finL.rotation.set(-0.9, 0.2, 0.5);
  finR.position.set(2.6, -0.9, -1.3);
  finR.rotation.set(0.9 + Math.PI, -0.2, 0.5);
  finR.scale.y = -1;
  const dorsal = new THREE.Mesh(new THREE.ExtrudeGeometry(shape([[0, 0], [-0.9, 0.55], [-1.2, 0]]), extrude), finMat);
  dorsal.position.set(-2.4, 0.95, -0.06);
  const eyeMat = new THREE.MeshStandardMaterial({ color: "#041018", roughness: 0.2 });
  const eyeL = new THREE.Mesh(new THREE.SphereGeometry(0.13, 10, 8), eyeMat), eyeR = eyeL.clone();
  eyeL.position.set(4.35, -0.15, 1.2);
  eyeR.position.set(4.35, -0.15, -1.2);
  const whale = new THREE.Group();
  whale.add(body, fluke, finL, finR, dorsal, eyeL, eyeR);
  whale.scale.setScalar(0.85);
  scene.add(whale);
  // Whales beat their tails up and down; the wave grows toward the tail.
  const wave = (x, t) => { const u = THREE.MathUtils.clamp((L / 2 - x) / L, 0, 1); return 0.55 * u * u * Math.sin(t * 1.6 - u * 3.2); };

  // ---- light rays
  const rayMat = new THREE.ShaderMaterial({
    transparent: true, depthWrite: false, blending: THREE.AdditiveBlending, side: THREE.DoubleSide,
    uniforms: { time: { value: 0 } },
    vertexShader: "varying vec2 vUv; void main(){ vUv = uv; gl_Position = projectionMatrix*modelViewMatrix*vec4(position,1.); }",
    fragmentShader: "uniform float time; varying vec2 vUv; void main(){ float edge = smoothstep(0.,.5,vUv.x)*smoothstep(1.,.5,vUv.x); float a = edge*pow(vUv.y,1.6)*(0.55+0.45*sin(time*.7+vUv.x*6.)); gl_FragColor = vec4(0.6,1.,0.95, a*0.07); }",
  });
  for (let i = 0; i < 9; i++) {
    const r = new THREE.Mesh(new THREE.PlaneGeometry(2 + Math.random() * 3, 40), rayMat);
    r.position.set(-18 + i * 4.5 + Math.random() * 2, 6, -8 - Math.random() * 10);
    r.rotation.z = -0.25 + Math.random() * 0.1;
    scene.add(r);
  }

  // ---- drifting particles ("marine snow")
  const dot = (() => {
    const c = document.createElement("canvas");
    c.width = c.height = 32;
    const g = c.getContext("2d"), gr = g.createRadialGradient(16, 16, 0, 16, 16, 16);
    gr.addColorStop(0, "rgba(255,255,255,1)");
    gr.addColorStop(1, "rgba(255,255,255,0)");
    g.fillStyle = gr;
    g.fillRect(0, 0, 32, 32);
    return new THREE.CanvasTexture(c);
  })();
  const N = 600, snow = new Float32Array(N * 3);
  for (let i = 0; i < N; i++) snow.set([(Math.random() - 0.5) * 60, 15 - Math.random() * 45, (Math.random() - 0.5) * 40 - 4], i * 3);
  const snowGeo = new THREE.BufferGeometry();
  snowGeo.setAttribute("position", new THREE.BufferAttribute(snow, 3));
  scene.add(new THREE.Points(snowGeo, new THREE.PointsMaterial({ size: 0.16, map: dot, transparent: true, opacity: 0.55, depthWrite: false, color: 0xbff7f0 })));

  // ---- fish school
  const fishGeo = new THREE.ConeGeometry(0.12, 0.5, 4);
  fishGeo.rotateZ(-Math.PI / 2);
  const fishes = new THREE.InstancedMesh(fishGeo, new THREE.MeshStandardMaterial({ color: "#ffd9a8", emissive: "#5a2e10", flatShading: true, roughness: 0.4 }), 36);
  const fishData = Array.from({ length: 36 }, () => ({ r: 3 + Math.random() * 2.5, y: (Math.random() - 0.5) * 2.5, s: 0.25 + Math.random() * 0.15, o: Math.random() * Math.PI * 2 }));
  scene.add(fishes);
  const m4 = new THREE.Matrix4(), q = new THREE.Quaternion(), v = new THREE.Vector3(), up = new THREE.Vector3(0, 1, 0), one = new THREE.Vector3(1, 1, 1);

  // ---- bubbles: a steady trickle, and a fountain when you click the water
  const B = 140, bub = new Float32Array(B * 3).fill(-999), bubVel = new Float32Array(B * 3), bubLife = new Float32Array(B).fill(1);
  const bubGeo = new THREE.BufferGeometry();
  bubGeo.setAttribute("position", new THREE.BufferAttribute(bub, 3));
  const bubbles = new THREE.Points(bubGeo, new THREE.PointsMaterial({ size: 0.3, map: dot, transparent: true, opacity: 0.75, depthWrite: false, color: 0xe6fffb }));
  bubbles.frustumCulled = false;
  scene.add(bubbles);
  let nextBub = 0;
  const emit = (at, speed, spread) => {
    const i = nextBub++ % B;
    bub.set([at.x, at.y, at.z], i * 3);
    bubVel.set([(Math.random() - 0.5) * spread, speed * (0.7 + Math.random() * 0.6), (Math.random() - 0.5) * spread], i * 3);
    bubLife[i] = 0;
  };

  // ---- memory trail: a glowing wake behind the tail
  const T = 90, trail = new Float32Array(T * 3), trailCol = new Float32Array(T * 3);
  const trailGeo = new THREE.BufferGeometry();
  trailGeo.setAttribute("position", new THREE.BufferAttribute(trail, 3));
  trailGeo.setAttribute("color", new THREE.BufferAttribute(trailCol, 3));
  const trailPts = new THREE.Points(trailGeo, new THREE.PointsMaterial({ size: 0.45, map: dot, vertexColors: true, transparent: true, depthWrite: false, blending: THREE.AdditiveBlending }));
  trailPts.frustumCulled = false;
  scene.add(trailPts);
  const glow = new THREE.Color("#2dd4bf");

  // ---- jellyfish, below the hero, seen as you scroll down
  const jellies = [];
  ["#7cf5e6", "#b58cff", "#ff9ec7", "#7cc8ff"].forEach((hex, n) => {
    for (let k = 0; k < 3; k++) {
      const col = new THREE.Color(hex), g = new THREE.Group();
      const glowMat = (opacity) => new THREE.MeshBasicMaterial({ color: col, transparent: true, opacity, blending: THREE.AdditiveBlending, depthWrite: false, side: THREE.DoubleSide });
      const bell = new THREE.Mesh(new THREE.SphereGeometry(0.8, 20, 10, 0, Math.PI * 2, 0, Math.PI / 2), glowMat(0.35));
      const core = new THREE.Mesh(new THREE.SphereGeometry(0.35, 12, 8), glowMat(0.5));
      core.position.y = 0.15;
      g.add(bell, core);
      const tentacles = [];
      for (let t = 0; t < 6; t++) {
        const line = new THREE.Line(
          new THREE.BufferGeometry().setFromPoints(Array.from({ length: 12 }, () => new THREE.Vector3())),
          new THREE.LineBasicMaterial({ color: col, transparent: true, opacity: 0.35, blending: THREE.AdditiveBlending, depthWrite: false })
        );
        line.userData.a = (t / 6) * Math.PI * 2;
        g.add(line);
        tentacles.push(line);
      }
      g.position.set((Math.random() - 0.5) * 30, -12 - Math.random() * 12, -4 - Math.random() * 10);
      g.scale.setScalar(0.6 + Math.random() * 0.7);
      g.userData = { home: g.position.clone(), phase: n * 3 + k * 1.7, bell, tentacles };
      scene.add(g);
      jellies.push(g);
    }
  });

  // ---- input: the whale leans toward the pointer; a click on open water makes it spout
  const clock = new THREE.Clock();
  let mx = 0, my = 0, lastMove = -99, spoutAt = -99;
  hero.addEventListener("pointermove", (e) => {
    const r = canvas.getBoundingClientRect();
    mx = (e.clientX - r.left) / r.width - 0.5;
    my = (e.clientY - r.top) / r.height - 0.5;
    lastMove = clock.getElapsedTime();
  });
  hero.addEventListener("click", (e) => {
    if (!e.target.closest("a, button, input, select, pre, figure")) spoutAt = clock.getElapsedTime();
  });

  const resize = () => {
    const { clientWidth: w, clientHeight: h } = canvas;
    if (!w || !h) return;
    renderer.setSize(w, h, false);
    camera.aspect = w / h;
    camera.updateProjectionMatrix();
  };
  new ResizeObserver(resize).observe(canvas);
  resize();

  const ray = new THREE.Raycaster(), plane = new THREE.Plane(new THREE.Vector3(0, 0, 1), 9), hit = new THREE.Vector3();
  const whalePos = new THREE.Vector3(-30, 1, 6), prev = whalePos.clone(), heading = new THREE.Vector3(1, 0, 0);
  const topShallow = new THREE.Color("#1a7a8c"), topDeep = new THREE.Color("#0a2f47");
  const fogShallow = new THREE.Color(0x062437), fogDeep = new THREE.Color(0x020b14);
  const cruise = (t, base) => new THREE.Vector3(Math.sin(t * 0.09) * 13, base + 4.6 + Math.sin(t * 0.21) * 0.9, Math.cos(t * 0.09) * 3 - 9);
  const intro = (t) => new THREE.Vector3(-30 + t * 6.5, 0.5 + Math.sin(t * 0.8) * 0.8, 7 - t * 1.2);
  let camY = 0, frameNo = 0, running = false;

  function frame() {
    const time = reduce ? 14 : clock.getElapsedTime();
    // Depth follows how far the hero has scrolled away: 0 in view, 1 gone.
    const rect = hero.getBoundingClientRect();
    const depth = THREE.MathUtils.clamp(-rect.top / Math.max(rect.height, 1), 0, 1);
    camY += (-depth * 14 - camY) * (reduce ? 1 : 0.06);

    // Whale: swims past the camera on arrival, then cruises; leans toward the pointer.
    const want = cruise(time, camY);
    if (!reduce) want.lerp(intro(time), 1 - THREE.MathUtils.smoothstep(time, 5.5, 9));
    if (time - lastMove < 3) {
      ray.setFromCamera({ x: mx * 2, y: -my * 2 }, camera);
      plane.constant = -want.z;
      if (ray.ray.intersectPlane(plane, hit)) want.lerp(hit, 0.35 * (1 - THREE.MathUtils.smoothstep(time - lastMove, 1.5, 3)));
    }
    prev.copy(whalePos);
    whalePos.lerp(want, reduce ? 1 : 0.035);
    const vel = whalePos.clone().sub(prev);
    vel.y *= 0.25; // keep the whale mostly level instead of nose-diving as the page scrolls
    if (vel.lengthSq() > 1e-6) heading.lerp(vel.normalize(), 0.08).normalize();
    whale.position.copy(whalePos);
    whale.lookAt(whalePos.clone().add(heading));
    whale.rotateY(-Math.PI / 2);
    const spouting = time - spoutAt < 1.6;
    if (spouting) whale.rotateX(Math.sin((time - spoutAt) * 4) * 0.12);

    const beat = time * (spouting ? 2.4 : 1);
    const p = bodyGeo.attributes.position;
    for (let i = 0; i < p.count; i++) p.array[i * 3 + 1] = basePos[i * 3 + 1] + wave(basePos[i * 3], beat);
    p.needsUpdate = true;
    bodyGeo.computeVertexNormals();
    const dy = wave(-L / 2, beat), slope = (wave(-L / 2 + 0.3, beat) - dy) / 0.3;
    fluke.position.set(-L / 2 + 0.1, dy, 0);
    fluke.rotation.z = Math.atan(slope) * 1.4;
    finL.rotation.x = -0.9 + Math.sin(time * 1.6) * 0.15;
    finR.rotation.x = 0.9 + Math.PI - Math.sin(time * 1.6) * 0.15;
    whale.updateMatrixWorld();

    const blowhole = new THREE.Vector3(3.2, 1.3, 0).applyMatrix4(whale.matrixWorld);
    if (frameNo % 9 === 0) emit(blowhole, 0.05, 0.02);
    if (spouting && time - spoutAt < 0.7) for (let k = 0; k < 4; k++) emit(blowhole, 0.22, 0.12);
    for (let i = 0; i < B; i++) {
      if (bubLife[i] >= 1) { bub[i * 3 + 1] = -999; continue; }
      bubLife[i] += 0.006;
      bubVel[i * 3 + 1] = Math.max(bubVel[i * 3 + 1] * 0.985, 0.03);
      bub[i * 3] += bubVel[i * 3] + Math.sin(time * 3 + i) * 0.004;
      bub[i * 3 + 1] += bubVel[i * 3 + 1];
      bub[i * 3 + 2] += bubVel[i * 3 + 2];
    }
    bubGeo.attributes.position.needsUpdate = true;

    const tail = new THREE.Vector3(-L / 2, dy, 0).applyMatrix4(whale.matrixWorld);
    trail.copyWithin(3, 0, (T - 1) * 3);
    trail.set([tail.x + (Math.random() - 0.5) * 0.3, tail.y + (Math.random() - 0.5) * 0.3, tail.z], 0);
    for (let i = 0; i < T; i++) {
      const f = Math.pow(1 - i / T, 2) * 0.8;
      trailCol.set([glow.r * f, glow.g * f, glow.b * f], i * 3);
    }
    trailGeo.attributes.position.needsUpdate = true;
    trailGeo.attributes.color.needsUpdate = true;

    for (const j of jellies) {
      const { home, phase, bell, tentacles } = j.userData, t = time + phase;
      const pulse = Math.pow(Math.sin(t * 1.3) * 0.5 + 0.5, 2);
      bell.scale.set(1 - pulse * 0.18, 1 + pulse * 0.12, 1 - pulse * 0.18);
      j.position.set(home.x + Math.sin(t * 0.2) * 1.5, home.y + Math.sin(t * 0.4) * 1.2 + pulse * 0.2, home.z);
      for (const line of tentacles) {
        const a = line.userData.a, pos = line.geometry.attributes.position;
        for (let k = 0; k < 12; k++) {
          pos.setXYZ(k, Math.cos(a) * 0.55 + Math.sin(t * 2 + k * 0.5 + a) * 0.08 * k, -k * 0.22, Math.sin(a) * 0.55 + Math.cos(t * 1.7 + k * 0.5) * 0.06 * k);
        }
        pos.needsUpdate = true;
      }
    }

    const s = snowGeo.attributes.position;
    for (let i = 0; i < N; i++) {
      s.array[i * 3 + 1] += 0.004;
      if (s.array[i * 3 + 1] > 15) s.array[i * 3 + 1] = -30;
    }
    s.needsUpdate = true;
    fishData.forEach((f, i) => {
      const a = time * f.s + f.o;
      v.set(-9 + Math.cos(a) * f.r * 1.4, -5.5 + f.y + Math.sin(a * 2) * 0.3, -8 + Math.sin(a) * f.r * 0.6);
      q.setFromAxisAngle(up, -a - Math.PI / 2);
      fishes.setMatrixAt(i, m4.compose(v, q, one));
    });
    fishes.instanceMatrix.needsUpdate = true;

    bg.material.uniforms.top.value.copy(topShallow).lerp(topDeep, depth);
    scene.fog.color.copy(fogShallow).lerp(fogDeep, depth);
    rayMat.uniforms.time.value = time;
    bg.material.uniforms.time.value = time;
    camera.position.x += (mx * 2.5 - camera.position.x) * 0.03;
    camera.position.y += (camY - my * 1.5 - camera.position.y) * (reduce ? 1 : 0.05);
    camera.lookAt(0, camY, 0);
    bg.position.copy(camera.position);

    renderer.render(scene, camera);
    frameNo++;
    if (running && !reduce) requestAnimationFrame(frame);
  }

  if (reduce) {
    frame();
    addEventListener("scroll", frame, { passive: true });
    return;
  }
  // Only animate while the hero is visible.
  new IntersectionObserver(([entry]) => {
    const wasRunning = running;
    running = entry.isIntersecting;
    if (running && !wasRunning) requestAnimationFrame(frame);
  }).observe(hero);
}
