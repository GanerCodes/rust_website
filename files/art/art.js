backgroundColor = '454577'
var clrs = [
    '#878a87',
    '#cbdbc8',
    '#e8e0d4',
    '#b29e91',
    '#9f736c',
    '#b76254',
    '#dfa372'
];



let limit = 10;
let s = 100;

let locList = {};
let blob, seed, base, img;

function rb() {
    return random(0, 1) >= 0.5;
}
function hexToRgb(hex) {
    let result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    return [parseInt(result[1], 16), parseInt(result[2], 16), parseInt(result[3], 16)]
}
clrs = clrs.map(x => hexToRgb(x))
backgroundColor = hexToRgb(backgroundColor);

function mod(n, m) { //because JS is s p e c i a l
    return ((n % m) + m) % m;
}

function anyShape(base, xIn, yIn, zA) {
    let strtAng = random(0, 90);
    let a = strtAng;
    base.beginShape();
    for (let i = 0; i <= 3; i++) {
        if (i == 1) a += (90 - strtAng) * 2 + random(-5, 5);
        else if (i == 2) a += 180 - (2 * (90 - strtAng)) + random(-5, 5);
        else if (i == 3) a += 2 * (90 - strtAng) + random(-5, 5);
        let x = xIn + cos(radians(a)) * zA;
        let y = yIn + sin(radians(a)) * zA;
        base.vertex(x, y);
    }
    base.endShape(CLOSE);
}

function makeShape(r, x, y, seed, limit, f, pq) {
    randomSeed(seed);
    
    let c1 = clrs[Math.floor(random(0, clrs.length))];
    let c2 = clrs[Math.floor(random(0, clrs.length))];
    // let c3 = clrs[Math.floor(random(0, clrs.length))];
    
    if(pq == undefined) {
        pq = [false, false, false, false];
    }
    if(x in locList) {
        if(locList[x].includes(y)) {
            if(random(0, 1) > 0.8) {
                x += s * Math.floor((-3, 3)) / 3;
                y += s * Math.floor((-3, 3)) / 3;
            }else{
                return;
            }
        }else{
            locList[x].push(y);
        }
    }else{
        locList[x] = [y];
    }
    let d = 1;
    let p = 1;
    let coords = [
        L  = [-d,  0], // L
        BL = [-p,  p], // BL
        B  = [ 0,  d], // B
        BR = [ p,  p], // BR
        R  = [ d,  0], // R
        TR = [ p, -p], // TR
        T  = [ 0, -d], // T
        TL = [-p, -p], // TL
    ]
    let q = [
        (!pq[0] && rb()),
        (!pq[1] && rb()),
        (!pq[2] && rb()),
        (!pq[3] && rb())
    ];
    let mw = s / 2;
    let mh = s / 2;
    
    blob.clear();
    blob.push();
    blob.translate(blob.width / 2, blob.height / 2);
    blob.rectMode(CENTER);
    blob.fill(255);
    blob.strokeWeight(3);
    blob.stroke(10, 10, 10);
    if(rb()) {
        blob.beginShape();
        blob.vertex(mw * coords[0][0], mh * coords[0][1]);
        for(let i = 1; i < coords.length; i += 2) {
            if(q[floor(0.5 * (i - 1))]) {
                if(random(0, 1) > 0.3) {
                    blob.quadraticVertex(
                        mw * coords[i % coords.length][0],
                        mh * coords[i % coords.length][1],
                        mw * coords[(i + 1) % coords.length][0],
                        mh * coords[(i + 1) % coords.length][1]
                    );
                }
            }else{
                if(random(0, 1) > 0.1) {
                    blob.vertex(
                        mw * coords[i % coords.length][0],
                        mh * coords[i % coords.length][1]
                    );
                }
                if(random(0, 1) > 0.1) {
                    blob.vertex(
                        mw * coords[(i + 1) % coords.length][0],
                        mh * coords[(i + 1) % coords.length][1]
                    );
                }
            }
        }
        blob.endShape(CLOSE);
    }else{
        anyShape(blob, 0, 0, s / 2);
    }
    blob.pop();
    blob.loadPixels();
    let img = createImage(blob.width, blob.height);
    img.loadPixels();
    
    let cond = Math.floor(random(0, 7)); //Pattern number
    let randAng = random(0, TWO_PI); //For pattens that take an angle
    let hbw = blob.width  / 2 + random(-1, 1); //Pattern offset X
    let hbh = blob.height / 2 + random(-1, 1); //Pattern offset Y
    let sx = 1.5 / blob.width ; //Pattern scaler X
    let sy = 1.5 / blob.height; //Pattern scaler Y
    let ix = 0, iy = 0; //Image coords
    let ax = 0, ay = 0; //Cartesian coords
    let v = 0; //Pattern eq result
    for(let i = 0; i < blob.pixels.length; i += 4) {
        if(blob.pixels[i] > 20) {
            img.pixels[i + 3] = 255;
            ax = sx * (ix - hbw);
            ay = sy * (iy - hbh);
            switch(cond) {
                case 0: {
                    v = Math.sin(5 * TWO_PI * Math.sqrt(pow(ax, 2) + pow(ay, 2)));
                } break;
                case 1: {
                    v = Math.sin(15 * ax) + Math.cos(15 * ay);
                } break;
                case 2: {
                    v = pow(Math.sin(12 * ax), 2) + pow(Math.cos(12 * ay), 2) - pow(0.8, 2);
                } break;
                case 3: {
                    v = sin(10 * Math.atan2(ay, ax));
                } break;
                case 4: {
                    v = cos(17 * (sin(randAng) * ay - cos(randAng) * ax));
                } break;
                case 5: {
                    let tx = mod(ax, 0.6) - 0.3;
                    let ty = mod(ay, 0.6) - 0.3;
                    v = abs(tx) + abs(ty) + max(abs(tx), abs(ty)) - 0.5;
                } break;
                case 6: {
                    v = cos(10 * ay) - 0.9 * sin(7 * ax);
                } break;
            }
            if(v <= 0) {
                img.pixels[i + 0] = c1[0];
                img.pixels[i + 1] = c1[1];
                img.pixels[i + 2] = c1[2];
                img.pixels[i + 3] = 255;
            }else{
                img.pixels[i + 0] = c2[0];
                img.pixels[i + 1] = c2[1];
                img.pixels[i + 2] = c2[2];
                img.pixels[i + 3] = 255;
            }
        }else if(blob.pixels[i + 1] > 0) {
            img.pixels[i + 0] = blob.pixels[i + 0];
            img.pixels[i + 1] = blob.pixels[i + 1];
            img.pixels[i + 2] = blob.pixels[i + 2];
            img.pixels[i + 3] = blob.pixels[i + 3];
        }
        
        ix++;
        if(ix == blob.width) {
            ix = 0;
            iy++;
        }
    }
    
    ix = 0, iy = 0;
    let r = 0, g = 0, b = 0;
    for(let i = 0; i < image.pixels.length; i += 4) {
        if(image.pixels[i + 0] + image.pixels[i + 1] + image.pixels[i + 2] < 1) {
            r = 0, g = 0, b = 0;
            if(ix > 0) {
                r +=
            }
        }
        ix++;
        if(ix == image.width) {
            ix = 0;
            iy++;
        }
    }

    
    img.updatePixels();
    
    r.push();
    r.translate(x, y);
    r.image(img, 0, 0);
    r.pop();
    if(limit > 0) {
        limit--;
        j = [];
        if(!(q[0] || q[1])) { j.push([-1,  0, 0]); }
        if(!(q[1] || q[2])) { j.push([ 0, -1, 1]); }
        if(!(q[2] || q[3])) { j.push([ 1,  0, 2]); }
        if(!(q[3] || q[0])) { j.push([ 0,  1, 3]); }
        for(let c of j) {
            let nX = x + c[0] * s;
            let nY = y + c[1] * s;
            makeShape(r, nX, nY, random(0, 100000000), limit, f + 1, [...q]);
        }
    }
    return q;
}

function setup() {
    var cnv = createCanvas(2500, 1280);
    cnv.style('display', 'block');
    
    smooth();
    
    rectMode(CENTER);
    imageMode(CENTER);
    
    seed = random(1, 1000000000);
    randomSeed(seed);
    
    base = createGraphics(width, height);
    blob = createGraphics(100, 100);
    
    base.drawingContext.shadowOffsetX = 5;
    base.drawingContext.shadowOffsetY = 5;
    base.drawingContext.shadowBlur = 15;
    base.drawingContext.shadowColor = 'black';
    
    base.smooth();
    blob.smooth();
    for(let i = 0; i < 14; i++) {
        let x1 = s * Math.floor(random(1/6 * base.width , 5/6 * base.width ) / s);
        let y1 = s * Math.floor(random(1/6 * base.height, 5/6 * base.height) / s);
        makeShape(base, x1, y1, random(0, 100000000), 10, 0); //base.width / 2, base.height / 2
    }
}
function draw() {
    background(backgroundColor[0], backgroundColor[1], backgroundColor[2]);
    
    image(base, width / 2, height / 2);
    
    strokeWeight(5);
    stroke(255);
    noFill();
    rect(width / 2, height / 2, width, height);
}
function keyPressed() {
    randomSeed(random(1, 1000000000));
    seed = random(1, 1000000000);
}