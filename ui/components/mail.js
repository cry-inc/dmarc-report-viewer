import { LitElement, html, css } from "lit";
import { globalStyle } from "./style.js";

export class Mail extends LitElement {
    static styles = [globalStyle, css`
        table {
            width: 100%;
            margin-bottom: 20px;
        }
    
        th {
            text-align: left;
            background-color: #efefef;
            width: 200px;
        }

        td, th {
            padding-left: 10px;
            padding-right: 10px;
            padding-top: 3px;
            padding-bottom: 3px;
        }

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
            const reportsResponse = await fetch("reports?uid=" + this.uid);
            this.reports = await reportsResponse.json();
            const errorsResponse = await fetch("mails/" + this.uid + "/errors");
            this.errors = await errorsResponse.json();
        }
    }

    renderOversized(oversized) {
        if (oversized) {
            return html`<span class="badge badge-negative">Yes</span>`;
        } else {
            return html`<span class="noproblem">No</span>`;
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
                    <th>UID</th>
                    <td>${this.mail.uid}</td>
                </tr>
                <tr>
                    <th>Size</th>
                    <td>${this.mail.size} Bytes</td>
                </tr>
                <tr>
                    <th>Oversized</th>
                    <td>${this.renderOversized(this.mail.oversized)}</td>
                </tr>
                <tr>
                    <th>Date</th>
                    <td>${new Date(this.mail.date * 1000).toLocaleString()}</td>
                </tr>
                <tr>
                    <th>Subject</th>
                    <td>${this.mail.subject}</td>
                </tr>
                <tr>
                    <th>Sender</th>
                    <td>
                        <a href="#/mails?sender=${encodeURIComponent(this.mail.sender)}">
                            ${this.mail.sender}
                        </a>
                    </td>
                </tr>
                <tr>
                    <th>Recipient</th>
                    <td>${this.mail.to}</td>
                </tr>
            </table>

            <h2>Reports extracted from this Mail</h2>
            <dmarc-report-table .reports="${this.reports}"></dmarc-report-table>

            ${this.errors.length > 0 ?
                html`
                    <h3>XML Parsing Errors</h3>
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
