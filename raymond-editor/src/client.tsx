import React, { useState, useEffect } from "react"
import ReactDOM from "react-dom"

import * as THREE from "three"
import { Canvas, useThree } from "react-three-fiber"
import { GridHelper } from "three";

import { css, Global } from "@emotion/core"

import {Scene, Sphere, Plane, Vector3} from "./model"

import "core-js/stable";
import "regenerator-runtime/runtime";

const STATES = {
    Default: 0,
    Rotating: 1,
    Translating: 2,
}

const dbg = console.log

const scene = new Scene();
scene.objects.push(new Plane(new Vector3(0.0, -1.0, 0.0), new Vector3(0.0, 1.0, 0.0), null));

console.log(JSON.stringify(scene, undefined, 4));

function SceneView() {

    const [state, setState] = useState(STATES.Default)
    const three = useThree()

    const onPointerDown = (e: any) => {
        if (e.button == 1 && e.shiftKey) {
            setState(STATES.Translating)
            e.target.setPointerCapture(e.pointerId)
        } else if (e.button == 1 ) {
            setState(STATES.Rotating)
            e.target.setPointerCapture(e.pointerId)
        }
    }

    const onPointerUp = (e: any) => {
        setState(STATES.Default)
        e.target.releasePointerCapture(e.pointerId)
    }

    const onPointerMove = (e: any) => {
        if (state == STATES.Rotating) {
            three.camera = three.camera.rotateOnWorldAxis(new THREE.Vector3(0, 1, 0), e.movementX / 4000.0)
            three.camera = three.camera.rotateOnAxis(new THREE.Vector3(1, 0, 0), e.movementY / 4000.0)
        }
        if (state == STATES.Translating) {
            three.camera.translateX(-e.movementX / 1000)
            three.camera.translateY(e.movementY / 1000)
        }
    }

    const onWheel = (e: any) => {
        three.camera.translateZ(-e.nativeEvent.wheelDelta/ 10000.0)
    }

    return (
        <group
            onPointerDown={onPointerDown}
            onPointerUp={onPointerUp}
            onPointerMove={onPointerMove}
            onWheel={onWheel}
        >
            <gridHelper userData={{ size: 1, divisions: 100 }}
            />
            <line>
                <geometry
                    attach="geometry"
                    vertices={([[-1, 0, 0], [0, 1, 0], [1, 0, 0], [0, -1, 0], [-1, 0, 0]]).map(v => new THREE.Vector3(...v))}
                    onUpdate={self => (self.verticesNeedUpdate = true)}
                />
            </line>
        </group>
    )
}

const globalStyle = css`
* {
    margin: 0;
    padding: 0;
}

html, body {
    width: 100%;
    height: 100%;
}

.app {
    width: 100%;
    height: 100%;
}
`

let image: ImageData | null = null

class OutputCanvas extends React.Component {
    componentDidMount() {
        let canvas = ReactDOM.findDOMNode(this.refs.myCanvas) as HTMLCanvasElement;
        let ctx = canvas.getContext("bitmaprenderer") as ImageBitmapRenderingContext;
        
        image = new ImageData(800, 600);

        for (let i = 0; i < 800; i++) {
            for(let b = 0; b < 600; b++) {
                image.data[4 * (i + b * 800)] = 255
                image.data[4 * (i + b * 800) + 3] = 255
            }
        }

        (async (image) => {
            const bitmap = await createImageBitmap(image);
            ctx.transferFromImageBitmap(bitmap);
            console.log("heyo");
        })(image);

    }

    render() {
        return (
            <canvas ref="myCanvas" />
        )
    }
}

function Editor() {
    return (
        <React.Fragment>
            <Global styles={globalStyle} />
            <Canvas>
                <SceneView />
            </Canvas>
            <OutputCanvas />
        </React.Fragment>
    )
}

ReactDOM.render(
    <Editor />,
    document.querySelector(".app")
)