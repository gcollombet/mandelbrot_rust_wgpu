// shader that renders a zoom deep into the mandelbrot set with minimal effort.
// works by storing the current position relative to a nearby minibrot, and
// switching between reference minibrots as needed. iterations are also skipped
// as a result, allowing this to compute even the millions of iterations occuring
// towards the later part of the zoom with relatively few actual computations.

// the final minibrot here has a period of around 647000 and occurs at a zoom of
// around 1e97, from which we zoom in further by a factor 1e6 or so.

const float BAILOUT = 4096.0;

struct Minibrot {
    vec2 position;
    vec2 scale;
    vec2 a;
    float approximationRadius;
    float period;
    float innerApproximationRadius;
};

vec2 cmul(vec2 a, vec2 b){
    return vec2(dot(a,vec2(1.0,-1.0)*b),dot(a,b.yx));
}

vec2 cdiv(vec2 a, vec2 b){
    return vec2(dot(a,b),dot(a.yx,vec2(1.0,-1.0)*b))/dot(b,b);
}

void mainImage(out vec4 fragColor, in vec2 fragCoord) {
    // minibrots that this zoom passes by, in order. generated using another (currently private) shader.
    // those values are all relative to the previous minibrot each, which is also why we got fractional periods occuring here.
    Minibrot[] minibrots = Minibrot[](
    Minibrot(vec2(0.0,0.0),vec2(1.0000000,0.0),vec2(1.0000000,0.0),1e310,1.0000000,1e310),
    Minibrot(vec2(-1.7548777,7.6738615e-13),vec2(0.019035511,8.2273679e-14),vec2(-9.2988729,1.7585520e-11),0.14589541,3.0000000,0.14539978),
    Minibrot(vec2(0.26577526,7.9269924e-14),vec2(1.7697619e-7,-5.1340973e-17),vec2(270.20074,4.0151175e-8),0.000077616154,24.666666,0.0063840700),
    Minibrot(vec2(-108.77785,-54.851582),vec2(-0.0066274544,-0.0015506834),vec2(-1.3899392,-12.041053),2.4771779,2.0540543,2.4771934),
    Minibrot(vec2(7.1870313,8.6428413),vec2(-0.0000022837414,-0.0000032153393),vec2(-220.36572,-444.80170),0.036942456,2.4868422,0.037870880),
    Minibrot(vec2(-1568.3745,271.39987),vec2(0.000062814426,-0.00053683209),vec2(32.133900,28.593172),1.9547913,1.8042328,1.9547923),
    Minibrot(vec2(-39.815723,-13.059175),vec2(2.0205009e-8,-1.0168816e-8),vec2(-6508.9990,-1532.6521),0.0095185349,2.5542521,0.0094131418),
    Minibrot(vec2(-36646.895,-15671.298),vec2(0.000033099699,-0.000025827576),vec2(-145.93997,-50.201008),3.9468935,1.7830081,3.9468935),
    Minibrot(vec2(45.757519,-169.32626),vec2(-3.0256565e-11,4.0970952e-11),vec2(62932.395,-125135.27),0.0021758196,2.5608499,0.0021774585),
    Minibrot(vec2(-1356.5258,127.47163),vec2(-1.4676038e-11,-1.9145330e-10),vec2(-49048.219,-52953.777),0.091265261,2.3904953,0.091265261),
    Minibrot(vec2(617.66748,-1510.6793),vec2(-1.3358331e-11,1.0865342e-11),vec2(80652.148,-227075.38),0.089992270,2.4183233,0.089992270),
    Minibrot(vec2(-3096.2500,389.08243),vec2(-5.4140579e-13,-3.8956390e-12),vec2(-331122.59,-380331.28),0.14324503,2.4135096,0.14324503),
    Minibrot(vec2(-4897.3506,-2047.1642),vec2(5.5859196e-13,-4.3974197e-13),vec2(1120745.9,388190.56),0.16162916,2.4143343,0.16162916),
    Minibrot(vec2(-8970.1211,-2440.9661),vec2(8.1748722e-14,-1.0900626e-13),vec2(-2423142.8,-1211603.1),0.17583868,2.4141929,0.17583868)
    );

    // relative to the innermost minibrot
    vec4 cameraPosition = vec4(-1.2517219,-0.2642476,-1.5803953e-7,1.3786521e-6);
    float zoomOutLog = max(0.0,104.5-0.4*iTime);

    vec2 dc = cmul(vec2(10.0/length(iResolution),0.0),cameraPosition.zw);
    vec2 c = cameraPosition.xy/pow(10.0,zoomOutLog)+cmul(dc,vec2(1.,-1.)*(fragCoord-iResolution.xy*0.5));

    int minibrotIndex = 13;
    Minibrot minibrot = minibrots[minibrotIndex];

    // some ugly trickery to avoid c and dc overflowing to infinity - instead multiplying the zoom factor
    // with them directly, it is gradually cancelled out with the innermost minibrot scales
    while(minibrotIndex>0&&zoomOutLog>20.0){
        c = minibrot.position/pow(10.0,zoomOutLog)+cmul(c,minibrot.scale);
        dc = cmul(dc,minibrot.scale);
        c /= length(minibrot.scale);
        dc /= length(minibrot.scale);
        zoomOutLog += log(length(minibrot.scale))/log(10.0);
        minibrotIndex--;
        minibrot = minibrots[minibrotIndex];
    }

    c *= pow(10.0,zoomOutLog);
    dc *= pow(10.0,zoomOutLog);

    // actual algorithm starts here.
    while(length(c.xy)>minibrot.approximationRadius/length(minibrot.scale)){
        c = minibrot.position+cmul(c,minibrot.scale);
        dc = cmul(dc,minibrot.scale);
        minibrotIndex--;
        minibrot = minibrots[minibrotIndex];
    }

    vec2 z = vec2(0.0);
    vec2 dz = vec2(0.0);
    float escapeRadius = minibrotIndex==0?BAILOUT:max(4.2,(minibrot.innerApproximationRadius)*dot(minibrot.a,minibrot.a));
    float iteration = 0.0;
    for (int i=0;i<20;i++){
        if (dot(z,z)>escapeRadius){
            if (minibrotIndex==0){
                break;
            }else{
                z = cdiv(z,minibrot.a);
                dz = cdiv(dz,minibrot.a);
                c = minibrot.position+cmul(c,minibrot.scale);
                dc = cmul(dc,minibrot.scale);
                minibrotIndex--;
                minibrot = minibrots[minibrotIndex];
                escapeRadius = minibrotIndex==0?BAILOUT:max(4.2,(minibrot.innerApproximationRadius)*dot(minibrot.a,minibrot.a));
            }
        }
        dz = 2.0*cmul(dz,z)+dc;
        z = cmul(z,z)+c;
        iteration++;
    }
    fragColor.xyz = vec3(max(0.0,log(log(dot(dz,dz)/dot(z,z)))));

}