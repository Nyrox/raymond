import {Plane} from "./plane"
import {Sphere} from "./sphere"

type SceneObject = Plane | Sphere


export class Scene {
    public objects: SceneObject[]

    constructor() {
        this.objects = []
    }
}