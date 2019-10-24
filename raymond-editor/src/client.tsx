import React, { useState, useEffect } from "react"
import ReactDOM, { findDOMNode } from "react-dom"

import * as THREE from "three"
import { Canvas, useThree } from "react-three-fiber"
import { GridHelper } from "three";

import { css, Global } from "@emotion/core"

import {Scene, Sphere, Plane, Vector3} from "./model"

import "core-js/stable";
import "regenerator-runtime/runtime";
import { Socket } from "net";

import * as assert from "assert"

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

const net = require("net")
const msgpack = require("msgpack-lite");

// samples, width, height, left, top
const HEADER_LENGTH = 5 * 4


interface Header {
    sample_count: number,
    width: number,
    height: number,
    left: number,
    top: number,
}

type MessageCallback = (header: Header, body: Float64Array) => void

class NetworkDecoder {
    bytes_held = 0
    queue: Buffer[]
    socket: Socket
    on_message: MessageCallback
    header: Header | null = null

    constructor(socket: Socket, on_message: MessageCallback) {
        this.socket = socket
        this.queue = []
        this.on_message = on_message

        this.socket.on("data", this._onData.bind(this))
        
    }

    readBytes(size: number): Uint8Array {
        if (size > this.bytes_held) throw new Error("[NetworkDecoder::readBytes] Requested more bytes than buffered")

        let outputBuffer = new Uint8Array(size)
        let buffered = 0
        let remaining = size

        while (remaining > 0) {
            let readAmount = Math.min(remaining, this.queue[0].length)

            for (let i = 0; i < readAmount; i++) {
                outputBuffer[buffered] = this.queue[0][i]
                buffered = buffered + 1
            }

            // If we extracted the whole array, we pop that buffer from the queue
            if (readAmount == this.queue[0].length) {
                this.queue = this.queue.slice(1)
            } 
            // Else we need to slice the first buffer to remove the part we read
            else {
                this.queue[0] = this.queue[0].slice(readAmount)
            }

            remaining = remaining - readAmount
        }

        this.bytes_held -= size
        return outputBuffer
    }
    
    readHeader(): Header {
        const bytes = this.readBytes(HEADER_LENGTH)
        const values = new Uint32Array(bytes.buffer)

        let [sample_count, width, height, left, top] = values
        return {
            sample_count,
            width,
            height,
            left,
            top
        }
    }

    readBody(): Float64Array {
        const header = this.header as Header
        // width * height * RGB * f64
        const bytes = this.readBytes(header.width * header.height * 3 * 8)
        const values = new Float64Array(bytes.buffer)

        return values
    }

    _onData(data: Buffer) {
        this.queue.push(data);
        this.bytes_held += data.length
    
        if (this.header == null) {
            if (this.bytes_held >= HEADER_LENGTH) {
                this.header = this.readHeader();
            }
        } else {
            const header = this.header as Header
            if (this.bytes_held >= header.width * header.height * 3 * 8) {
                let body = this.readBody();
                this.on_message(this.header, body)
                this.header = null
            }
        }
    }    
}


class OutputCanvas extends React.Component {
    componentDidMount() {
        let canvas = ReactDOM.findDOMNode(this.refs.myCanvas) as HTMLCanvasElement;
        let ctx = canvas.getContext("2d") as CanvasRenderingContext2D

        const conn = net.createConnection({
            port: 17025,
            host: "localhost",
        })

        let decoder = new NetworkDecoder(conn, (header, body) => {
            let colorData = body
            
            // our incoming data is RGB, but we need to provide RGBA
            const mappedData = new Uint8ClampedArray(colorData.length / 3 * 4)

            const gamma = 2.2;
            const exposure = 1.0;

            for (let y = 0; y < header.height; y++) {
                for (let x = 0; x < header.width; x++) {
                    mappedData[4 * (x + y * header.width) + 0] = Math.pow((1 - Math.exp(colorData[3 * (x + y * header.width) + 0] * -1.0 * exposure)), 1 / gamma) * 255;
                    mappedData[4 * (x + y * header.width) + 1] = Math.pow((1 - Math.exp(colorData[3 * (x + y * header.width) + 1] * -1.0 * exposure)), 1 / gamma) * 255;
                    mappedData[4 * (x + y * header.width) + 2] = Math.pow((1 - Math.exp(colorData[3 * (x + y * header.width) + 2] * -1.0 * exposure)), 1 / gamma) * 255;
                    mappedData[4 * (x + y * header.width) + 3] = 255
                }
            }

            ctx.putImageData(new ImageData(mappedData, header.width, header.height), header.left, header.top)
        })

        // conn.on("data", async (data: any) => {
            
        //     const decoded = msgpack.decode(data)
            
        //     let [sample_count, width, height, left, top, colors] = decoded;

        //     const binaryData = new Uint8Array(colors)
        //     const colorData = new Float64Array(binaryData.buffer)


        // });
    }

    render() {
        return (
            <canvas width="1280" height="720" ref="myCanvas" />
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