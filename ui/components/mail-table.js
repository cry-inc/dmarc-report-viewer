import { LitElement, html } from "lit";
import { globalStyle } from "../style.js";

export class MailTable extends LitElement {
    static styles = [globalStyle];

    static properties = {
        mails: { type: Array },
    };

    constructor() {
        super();
        this.mails = [];
    }

    prepareSubject(subject) {
        subject = subject.replace(/Report Domain: |Report domain: /, "D: ");
        subject = subject.replace(/Submitter: /, "S: ");
        subject = subject.replace(/Report-ID: /, "ID: ");
        subject = subject.replace(/T00\.00\.00Z/, "");

        const limit = 70;
        if (subject.length <= limit) {
            return subject;
        } else {
            return subject.substring(0, limit) + "...";
        }
    }

    prepareSize(mail) {
        if (mail.oversized) {
            return html`<span class="badge badge-negative">${mail.size}</span>`;
        } else {
            return mail.size;
        }
    }

    prepareReportType(mail) {
        if (mail.oversized) {
            return html`<span class="faded">n/a</span>`;
        } else if (mail.xml_files < 1 && mail.json_files < 1) {
            return html`<span class="badge badge-negative">None</span>`;
        } else {
            const files = [];
            if (mail.xml_files > 0) files.push("DMARC");
            if (mail.json_files > 0) files.push("TLS");
            return html`<span class="faded">${files.join(", ")}</span>`;
        }
    }

    prepareParsingError(mail) {
        if (mail.oversized) {
            return html`<span class="faded">n/a</span>`;
        } else if (mail.xml_parsing_errors > 0 || mail.json_parsing_errors > 0) {
            return html`<span class="badge badge-negative">Yes</span>`;
        } else {
            return html`<span class="faded">No</span>`;
        }
    }

    prepareDuplicates(mail) {
        if (mail.oversized) {
            return html`<span class="faded">n/a</span>`;
        } else if (mail.dmarc_duplicates.length > 0 || mail.tls_duplicates.length) {
            return html`<span class="badge badge-warning">Yes</span>`;
        } else {
            return html`<span class="faded">No</span>`;
        }
    }

    render() {
        return html`
            <table>
                <tr>
                    <th class="help" title="Subject might be incomplete! Check details for full mail subject.">Subject</th>
                    <th class="sm-hidden">Sender</th>
                    <th class="md-hidden">Date</th>
                    <th class="xs-hidden help" title="Size of E-Mail in Bytes">Size</th>
                    <th class="md-hidden help" title="Type of reports in the Mail">Type</th>
                    <th class="lg-hidden help" title="Duplicated reports found in Mail?">Duplicates</th>
                    <th class="xs-hidden help" title="Did the mail cause parsing errors?">Errors</th>
                </tr>
                ${this.mails.length !== 0 ? this.mails.map((mail) =>
                    html`<tr> 
                        <td><a href="#/mails/${mail.id}">${this.prepareSubject(mail.subject)}</a></td>
                        <td class="sm-hidden"><a href="#/mails?sender=${encodeURIComponent(mail.sender)}">${mail.sender}</a></td>
                        <td class="md-hidden">${new Date(mail.date * 1000).toLocaleString()}</td>
                        <td class="xs-hidden">${this.prepareSize(mail)}</td>
                        <td class="md-hidden">${this.prepareReportType(mail)}</td>
                        <td class="lg-hidden">${this.prepareDuplicates(mail)}</td>
                        <td class="xs-hidden">${this.prepareParsingError(mail)}</td>
                    </tr>`
                ) : html`<tr>
                        <td colspan="6">No mails found.</td>
                    </tr>`
                }
            </table>
        `;
    }
}

customElements.define("drv-mail-table", MailTable);
