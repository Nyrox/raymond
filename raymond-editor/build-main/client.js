"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (Object.hasOwnProperty.call(mod, k)) result[k] = mod[k];
    result["default"] = mod;
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
const react_1 = __importDefault(require("react"));
const react_dom_1 = __importDefault(require("react-dom"));
const THREE = __importStar(require("three"));
const react_three_fiber_1 = require("react-three-fiber");
function SceneView() {
    return (react_1.default.createElement(react_three_fiber_1.Canvas, null,
        react_1.default.createElement("line", null,
            react_1.default.createElement("geometry", { attach: "geometry", vertices: ([[-1, 0, 0], [0, 1, 0], [1, 0, 0], [0, -1, 0], [-1, 0, 0]]).map(v => new THREE.Vector3(...v)), onUpdate: self => (self.verticesNeedUpdate = true) }))));
}
function Editor() {
    return (react_1.default.createElement("div", null,
        react_1.default.createElement(SceneView, null),
        react_1.default.createElement("p", null, "Hello World!")));
}
react_dom_1.default.render(react_1.default.createElement(Editor, null), document.querySelector(".app"));
