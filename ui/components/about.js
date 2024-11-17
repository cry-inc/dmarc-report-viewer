import { LitElement, html } from "lit";

export class About extends LitElement {
    static properties = {
        version: { type: Object },
        hash: { type: String },
        ref: { type: String }
    };

    constructor() {
        super();
        this.version = "n/a";
        this.hash = "n/a";
        this.ref = "n/a";
        this.updateBuild();
    }

    async updateBuild() {
        const versionResponse = await fetch("build");
        const json = await versionResponse.json();
        this.version = json.version;
        this.ref = json.ref;
        this.hash = json.hash;
    }

    render() {
        return html`
            <p>
                This DMARC Report Viewer is an open source application.<br>
                You can find the source code and license on Github:
                <a href="https://github.com/cry-inc/dmarc-report-viewer" target="_blank">github.com/cry-inc/dmarc-report-viewer</a>.
            </p>
            <p>
                Version: <b>${this.version}</b><br>
                Git Hash: <b>${this.hash}</b><br>
                Git Ref: <b>${this.ref}</b>
            </p>
        `;
    }
}

customElements.define("dmarc-about", About);
