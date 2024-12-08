import { LitElement, html, css } from "lit";

export class Mails extends LitElement {
    static properties = {
        params: { type: Object },
        mails: { type: Array },
    };

    constructor() {
        super();
        this.params = {};
        this.mails = [];
        this.filtered = false;
    }

    updated(changedProperties) {
        if (changedProperties.has("params")) {
            this.updateMails();
        }
    }

    async updateMails() {
        const urlParams = [];
        if (this.params.oversized === "true" || this.params.oversized === "false") {
            urlParams.push("oversized=" + this.params.oversized);
        }
        if (this.params.sender) {
            urlParams.push("sender=" + encodeURIComponent(this.params.sender));
        }
        if (this.params.count) {
            urlParams.push("count=" + encodeURIComponent(this.params.count));
        }
        let url = "mails";
        if (urlParams.length > 0) {
            url += "?" + urlParams.join("&");
        }
        const mailsResponse = await fetch(url);
        this.mails = await mailsResponse.json();
        this.mails.sort((a, b) => b.date - a.date);
        this.filtered = this.filtered = urlParams.length > 0;
    }

    render() {
        return html`
            <p>
                ${this.filtered ?
                    html`<a href="#/mails">Show all Mails</a>` :
                    html`<a href="#/mails?oversized=true">Show only Oversize Mails</a> | <a href="#/mails?count=0">Show only Mails without valid XML Files</a>`
                }
            </p>
            <dmarc-mail-table .mails="${this.mails}"></dmarc-mail-table>
        `;
    }
}

customElements.define("dmarc-mails", Mails);
