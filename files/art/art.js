function rb() {
    return random(0, 1) >= 0.5;
}

let locList = {};
let limit = 10;

function makeShape(r, x, y, seed, limit) {
    if(x in locList) {
        locList[x].push(y);
    }else{
        locList[x] = [y];
    }
    let s = 100;
    randomSeed(seed);
    r.push();
    r.translate(x, y);
    r.rectMode(CENTER);
    r.fill(255);
    r.noStroke();
    // let p = random(1, sqrt(2) * 0.95) * 0.9;
    let d = 1;
    let p = 1;//sqrt(2);
    let coords = [
        [-d,  0],
        [-p,  p],
        [ 0,  d],
        [ p,  p],
        [ d,  0],
        [ p, -p],
        [ 0, -d],
        [-p, -p]
    ]
    let mw = s / 2;//r.width  / 2;
    let mh = s / 2;//r.height / 2;
    r.beginShape();
    r.vertex(mw * coords[0][0], mh * coords[0][1]);
    let q = [rb(), rb(), rb(), rb()];
    for(let i = 1; i < coords.length; i += 2) {
        if(q[floor((i - 1) / 2)]) {
            r.quadraticVertex(
                mw * coords[i % coords.length][0],
                mh * coords[i % coords.length][1],
                mw * coords[(i + 1) % coords.length][0],
                mh * coords[(i + 1) % coords.length][1]
            );
        }else{
            r.vertex(
                mw * coords[i % coords.length][0],
                mh * coords[i % coords.length][1]
            );
            r.vertex(
                mw * coords[(i + 1) % coords.length][0],
                mh * coords[(i + 1) % coords.length][1]
            );
        }
    }
    r.endShape(CLOSE);
    r.pop();
    if(limit > 0) {
        limit--;
        j = []
        if(!q[2]) { j.push([-1,  0]); }
        if(!q[3]) { j.push([ 0, -1]); }
        if(!q[4]) { j.push([ 1,  0]); }
        if(!q[5]) { j.push([ 0,  1]); }
        for(let c of j) {
            let nX = x + c[0] * s;
            let nY = y + c[1] * s;
            if(nX in locList) {
                if(nY in locList[nX]) {
                    continue;
                }else{
                    locList[nX].push(nY);
                }
            }else{
                locList[nX] = [nxt][1];
            }
            makeShape(r, nX, nY, random(0, 100000000), limit);
        }
    }
    return q;
}

var seed;
var base, img;
var shapeShader;
function preload() {
    shapeShader = loadShader("shader.vert", "shader.frag");
}
function setup() {
    var cnv = createCanvas(1900, 920, WEBGL);
    cnv.style('display', 'block');
    seed = random(1, 1000000000);
    randomSeed(seed);
    base = createGraphics(width, height, WEBGL);
    base.fill(255, 0, 0);
    base.ellipse(0, 0, 50, 50);
    makeShape(base, 0, 0, seed, 4); //base.width / 2, base.height / 2
    // img = createImage(base.width, base.height);
    // img.copy(base, 0, 0, base.width, base.height, 0, 0, base.width, base.height);
}
function draw() {
    background(0);
    strokeWeight(5);
    stroke(255);
    noFill();
    rectMode(CENTER);
    rect(0, 0, width, height);
    
    imageMode(CENTER);
    // image(img, 0, 0);
    image(base, 0, 0);
    // background(base);
    
    fill(255, 0, 0);
    noStroke();
    circle(0, 0, 5);
}
function keyPressed() {
    randomSeed(random(1, 1000000000));
    seed = random(1, 1000000000);
}