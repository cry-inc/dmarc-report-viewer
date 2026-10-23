import { LitElement, html, css } from "lit";

export class TimeRange extends LitElement {
    static properties = {
        before: { type: Number, reflect: true },
        after: { type: Number, reflect: true },
    };

    static styles = css`
        :host {
            display: block;
        }

        dialog {
            border: none;
            border-radius: 5px;
            padding: 40px;
        }

        dialog::backdrop {
            background: rgb(0 0 0 / 70%);
        }

        h2 {
            margin-top: 0;
        }

        label {
            display: block;
        }

        .dialog-actions {
            display: flex;
            justify-content: flex-end;
            gap: 8px;
            margin-top: 24px;
        }
    `;

    constructor() {
        super();
        this.before = undefined;
        this.after = undefined;
    }

    toLocalInput(timestamp) {
        if (timestamp === undefined || timestamp === null || Number.isNaN(timestamp)) return "";
        const date = new Date(timestamp * 1000);
        const localDate = new Date(date.getTime() - date.getTimezoneOffset() * 60_000);
        return localDate.toISOString().slice(0, 16);
    }

    open() {
        return new Promise((resolve) => {
            this.resolve = resolve;
            document.body.append(this);
            this.updateComplete.then(() => this.renderRoot.querySelector("dialog").showModal());
        });
    }

    finish(result) {
        const dialog = this.renderRoot.querySelector("dialog");
        if (dialog.open) dialog.close();
        this.remove();
        this.resolve(result);
    }

    apply(event) {
        event.preventDefault();
        const form = event.currentTarget;
        const timestamp = (value) => value ? new Date(value).getTime() / 1000 : undefined;
        this.finish({
            after: timestamp(form.elements.namedItem("after").value),
            before: timestamp(form.elements.namedItem("before").value),
        });
    }

    cancel(event) {
        event.preventDefault();
        this.finish(undefined);
    }

    render() {
        return html`
            <dialog @cancel=${this.cancel}>
                <form @submit=${this.apply}>
                    <h2>Select Time Range</h2>
                    <label>Start: <input name="after" type="datetime-local" .value=${this.toLocalInput(this.after)}></label>
                    <label>End: <input name="before" type="datetime-local" .value=${this.toLocalInput(this.before)}></label>
                    <div class="dialog-actions">
                        <button type="button" @click=${() => this.finish(undefined)}>Cancel</button>
                        <button type="submit">Apply</button>
                    </div>
                </form>
            </dialog>
        `;
    }
}

customElements.define("drv-time-range", TimeRange);

// Opens a time range dialog and resolves with the selected dates, or undefined if cancelled.
export function timeRangeDialog(before, after) {
    const dialog = new TimeRange();
    dialog.before = before ? before.getTime() / 1000 : undefined;
    dialog.after = after ? after.getTime() / 1000 : undefined;
    return dialog.open().then((result) => result && ({
        after: result.after === undefined ? undefined : new Date(result.after * 1000),
        before: result.before === undefined ? undefined : new Date(result.before * 1000),
    }));
}
