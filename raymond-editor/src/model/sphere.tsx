import {Vector3} from "./cgmath/Vector3"

export class Sphere {
    public origin: Vector3
    public radius: Number

    constructor(origin: Vector3, radius: Number) {
        this.origin = origin
        this.radius = radius
    }
}