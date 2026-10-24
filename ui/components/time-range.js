import { LitElement, html, css } from "lit";
import { globalStyle } from "../style.js";

class TimeRange extends LitElement {
    static properties = {
        before: { type: Number, reflect: true },
        after: { type: Number, reflect: true },
        invalidRange: { state: true },
        rangeComplete: { state: true },
    };

    static styles = [globalStyle, css`
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

        input[type="datetime-local"] {
            box-sizing: border-box;
            display: block;
            width: 100%;
        }

        .range-field {
            margin-bottom: 40px;
        }

        .quickselects {
            display: flex;
            flex-wrap: wrap;
            gap: 4px;
            margin-top: 4px;
        }

        .quickselect-label {
            display: inline-flex;
            align-items: center;
        }

        .quickselects .button {
            margin: 0;
        }

        .range-warning {
            color: rgb(220, 53, 69);
            margin: 0 0 30px;
        }

        .dialog-actions {
            display: flex;
            justify-content: center;
            gap: 8px;
        }
    `];

    constructor() {
        super();
        this.before = undefined;
        this.after = undefined;
        this.invalidRange = false;
        this.rangeComplete = false;
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
            this.updateComplete.then(() => {
                this.updateRangeValidity();
                this.updateComplete.then(() => this.renderRoot.querySelector("dialog").showModal());
            });
        });
    }

    finish(result) {
        const dialog = this.renderRoot.querySelector("dialog");
        if (dialog.open) dialog.close();
        this.remove();
        this.resolve(result);
    }

    quickSelectButtons(inputName) {
        const options = [
            { label: "1d", days: 1 },
            { label: "3d", days: 3 },
            { label: "1w", days: 7 },
            { label: "2w", days: 14 },
            { label: "1m", months: 1 },
            { label: "3m", months: 3 },
            { label: "6m", months: 6 },
            { label: "1y", months: 12 },
        ];
        return html`
            ${options.map((option) => html`
                <button class="button sm" type="button" @click=${() => this.setRelativeDate(inputName, option)}>${option.label}</button>
            `)}
            <span class="quickselect-label">ago</span>
        `;
    }

    setRelativeDate(inputName, option) {
        const date = new Date();
        if (option.days) {
            date.setDate(date.getDate() - option.days);
        } else {
            const dayOfMonth = date.getDate();
            date.setDate(1);
            date.setMonth(date.getMonth() - option.months);
            const lastDayOfMonth = new Date(date.getFullYear(), date.getMonth() + 1, 0).getDate();
            date.setDate(Math.min(dayOfMonth, lastDayOfMonth));
        }
        const input = this.renderRoot.querySelector(`[name="${inputName}"]`);
        input.value = this.toLocalInput(date.getTime() / 1000);
        this.updateRangeValidity();
    }

    updateRangeValidity(form = this.renderRoot.querySelector("form")) {
        const after = form.elements.namedItem("after").value;
        const before = form.elements.namedItem("before").value;
        this.rangeComplete = Boolean(after || before);
        this.invalidRange = Boolean(after && before && new Date(after) >= new Date(before));
    }

    apply(event) {
        event.preventDefault();
        const form = event.currentTarget;
        this.updateRangeValidity(form);
        if (this.invalidRange) return;
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
                    <div class="range-field">
                        <label>Start: <input name="after" type="datetime-local" .value=${this.toLocalInput(this.after)} @input=${() => this.updateRangeValidity()}></label>
                        <div class="quickselects">
                            ${this.quickSelectButtons("after")}
                        </div>
                    </div>
                    <div class="range-field">
                        <label>End: <input name="before" type="datetime-local" .value=${this.toLocalInput(this.before)} @input=${() => this.updateRangeValidity()}></label>
                        <div class="quickselects">
                            ${this.quickSelectButtons("before")}
                        </div>
                    </div>
                    ${this.invalidRange ? html`<p class="range-warning" role="alert">Error: Start must be before end date!</p>` : ""}
                    <div class="dialog-actions">
                        <button class="button" type="button" @click=${() => this.finish(undefined)}>Cancel</button>
                        <button class="button" type="submit" ?disabled=${!this.rangeComplete || this.invalidRange}>Apply</button>
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
