import { LitElement, html, css } from "lit";

export class Problems extends LitElement {
    static styles = css`
        h1 {
            font-size: 20px;
        }

        pre {
            border: 1px solid #e0e0e0;
            border-radius: 3px;
            background-color: #efefef;
            padding: 5px;
        }

        .problem {
            margin-bottom: 50px;
        }
    `;

    static properties = {
        xmlErrors: { type: Array },
        oversizedMails: { type: Array },
    };

    constructor() {
        super();
        this.xmlErrors = [];
        this.oversizedMails = [];
        this.updateProblems();
    }

    async updateProblems() {
        const xmlResponse = await fetch("xml-errors");
        this.xmlErrors = await xmlResponse.json();
        const mailsResponse = await fetch("mails");
        const mails = await mailsResponse.json();
        this.oversizedMails = mails.filter((m) => m.oversized);
    }

    render() {
        return html`
            <h1>Oversized Mails</h1>
            ${this.oversizedMails.length == 0 ?
                html`<p class="problem">No oversized mails found.</p>` :
                html`<div class="problem"><dmarc-mail-table .mails="${this.oversizedMails}"></dmarc-mail-table></div>`}

            <h1>XML Parsing Errors</h1>
            ${this.xmlErrors.length == 0 ? html`<p class="problem">No XML parsing errors found.</p>` : html``}
            ${this.xmlErrors.map((e) =>
            html`
                <div class="problem">
                    ${e.error}
                    <pre>${e.xml}</pre>
                </div>`
            )}
        `;
    }
}

customElements.define("dmarc-problems", Problems);
