export default class StateWidgetType {
    static readonly LAUNCH = new StateWidgetType("Launch", "launch-grid", "green", "rocket_launch");
    static readonly ABORT = new StateWidgetType("Abort", "abort-grid", "red", "cancel");
    static readonly CUT = new StateWidgetType("Cut", "cut-grid", "blue", "cut");

    public readonly name: string;
    public readonly gridPosition: string;
    public readonly backgroundColor: string;
    public readonly iconName: string;

    constructor(name: string, gridPosition: string, backgroundColor: string, iconName: string) {
        this.name = name;
        this.gridPosition = gridPosition;
        this.backgroundColor = backgroundColor;
        this.iconName = iconName;
    }
}