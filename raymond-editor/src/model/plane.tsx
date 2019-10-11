import {Vector3} from "./cgmath/Vector3"


export class Plane {
    public origin: Vector3
    public normal: Vector3
    public radius: Number | null
    
    private type: String = "Plane"

    constructor(origin: Vector3, normal: Vector3, radius: Number | null) {
        this.origin = origin
        this.normal = normal
        this.radius = radius
    }
}