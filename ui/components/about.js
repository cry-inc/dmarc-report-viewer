import { LitElement, html } from "lit";

export class About extends LitElement {
    constructor() {
        super();
    }

    render() {
        return html`
            <div>
                This DMARC Report Viewer is an open source application.<br>
                You can find the source code and license on Github:
                <a href="https://github.com/cry-inc/dmarc-report-viewer" target="_blank">github.com/cry-inc/dmarc-report-viewer</a>.
            </div>
        `;
    }
}

customElements.define("dmarc-about", About);
