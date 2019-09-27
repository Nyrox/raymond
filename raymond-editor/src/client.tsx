import React, { useState, useEffect } from "react"
import ReactDOM from "react-dom"

import * as THREE from "three"
import { Canvas, useThree } from "react-three-fiber"
import { GridHelper } from "three";

import { css, Global } from "@emotion/core"

const STATES = {
    Default: 0,
    Rotating: 1,
    Translating: 2,
}

const dbg = console.log

function SceneView() {

    const [state, setState] = useState(STATES.Default)
    const three = useThree()

    const onPointerDown = e => {
        if (e.button == 1 && e.shiftKey) {
            setState(STATES.Translating)
            e.target.setPointerCapture(e.pointerId)
        } else if (e.button == 1 ) {
            setState(STATES.Rotating)
            e.target.setPointerCapture(e.pointerId)
        }
    }

    const onPointerUp = e => {
        setState(STATES.Default)
        e.target.releasePointerCapture(e.pointerId)
    }

    const onPointerMove = e => {
        if (state == STATES.Rotating) {
            three.camera = three.camera.rotateOnWorldAxis(new THREE.Vector3(0, 1, 0), e.movementX / 4000.0)
            three.camera = three.camera.rotateOnAxis(new THREE.Vector3(1, 0, 0), e.movementY / 4000.0)
        }
        if (state == STATES.Translating) {
            three.camera.translateX(-e.movementX / 1000)
            three.camera.translateY(e.movementY / 1000)
        }
    }

    const onWheel = e => {
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

function Editor() {
    return (
        <React.Fragment>
            <Global styles={globalStyle} />
            <Canvas>
                <SceneView />
            </Canvas>
            </React.Fragment>
    )
}

ReactDOM.render(
    <Editor />,
    document.querySelector(".app")
)