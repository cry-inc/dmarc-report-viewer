import { LitElement, html, css } from "lit";
import { globalStyle } from "./style.js";

export class Mail extends LitElement {
    static styles = [globalStyle, css`
        .error pre {
            border: 1px solid #e0e0e0;
            border-radius: 3px;
            background-color: #efefef;
            padding: 5px;
        }
    `];

    static get properties() {
        return {
            uid: { type: String },
            mail: { type: Object, attribute: false },
            reports: { type: Array, attribute: false },
            errors: { type: Array, attribute: false }
        };
    }

    constructor() {
        super();
        this.uid = null;
        this.mail = null;
        this.reports = [];
        this.errors = [];
    }

    async updated(changedProperties) {
        if (changedProperties.has("uid") && changedProperties.uid !== this.uid && this.uid) {
            const mailsResponse = await fetch("mails/" + this.uid);
            this.mail = await mailsResponse.json();
            const reportsResponse = await fetch("dmarc-reports?uid=" + this.uid);
            this.reports = await reportsResponse.json();
            const errorsResponse = await fetch("mails/" + this.uid + "/errors");
            this.errors = await errorsResponse.json();
        }
    }

    renderOversized(oversized) {
        if (oversized) {
            return html`<span class="badge badge-negative">Yes</span>`;
        } else {
            return html`<span class="faded">No</span>`;
        }
    }

    render() {
        if (!this.mail) {
            return html`No mail loaded`;
        }

        return html`
            <h1>Mail Details</h1>
            <table>
                <tr>
                    <td class="name">UID</td>
                    <td>${this.mail.uid}</td>
                </tr>
                <tr>
                    <td class="name">Size</td>
                    <td>${this.mail.size} Bytes</td>
                </tr>
                <tr>
                    <td class="name">Oversized</td>
                    <td>${this.renderOversized(this.mail.oversized)}</td>
                </tr>
                <tr>
                    <td class="name">Date</td>
                    <td>${new Date(this.mail.date * 1000).toLocaleString()}</td>
                </tr>
                <tr>
                    <td class="name">Subject</td>
                    <td>${this.mail.subject}</td>
                </tr>
                <tr>
                    <td class="name">Sender</td>
                    <td>
                        <a href="#/mails?sender=${encodeURIComponent(this.mail.sender)}">
                            ${this.mail.sender}
                        </a>
                    </td>
                </tr>
                <tr>
                    <td class="name">Recipient</td>
                    <td>${this.mail.to}</td>
                </tr>
            </table>

            <h2>DMARC Reports</h2>
            <dmarc-report-table .reports="${this.reports}"></dmarc-report-table>

            ${this.errors.length > 0 ?
                html`
                    <h2>XML Parsing Errors</h2>
                    ${this.errors.map((e) =>
                    html`
                        <div class="error">
                            ${e.error}
                            <pre>${e.xml}</pre>
                        </div>`
                    )}`
                : html``
            }
        `;
    }
}

customElements.define("dmarc-mail", Mail);
