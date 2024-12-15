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
        const queryParams = [];
        if (this.params.oversized === "true" || this.params.oversized === "false") {
            queryParams.push("oversized=" + this.params.oversized);
        }
        if (this.params.sender) {
            queryParams.push("sender=" + encodeURIComponent(this.params.sender));
        }
        if (this.params.count) {
            queryParams.push("count=" + encodeURIComponent(this.params.count));
        }
        if (this.params.errors === "true" || this.params.errors === "false") {
            queryParams.push("errors=" + this.params.errors);
        }
        let url = "mails";
        if (queryParams.length > 0) {
            url += "?" + queryParams.join("&");
        }
        const mailsResponse = await fetch(url);
        this.mails = await mailsResponse.json();
        this.mails.sort((a, b) => b.date - a.date);
        this.filtered = this.filtered = queryParams.length > 0;
    }

    render() {
        return html`
            <p>
                ${this.filtered ?
                    html`Filter active! Go back and <a href="#/mails">Show all Mails</a>` :
                    html`<a href="#/mails?oversized=true">Show Oversized Mails</a> |
                         <a href="#/mails?count=0&oversized=false">Show Mails without XML Files</a> |
                         <a href="#/mails?errors=true">Show Mails with XML Parsing Errors</a>`
            }
            </p>
            <dmarc-mail-table .mails="${this.mails}"></dmarc-mail-table>
        `;
    }
}

customElements.define("dmarc-mails", Mails);
