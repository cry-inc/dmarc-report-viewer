import { timeRangeDialog } from "./components/time-range.js";

export function join(elements, joiner) {
    return elements.flatMap(x => [joiner, x]).slice(1);
}

/// Download some data in memory as a file
export function downloadFile(fileName, fileContent, mimeType) {
    const blob = new Blob([fileContent], { type: mimeType });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = fileName;
    document.body.appendChild(link);
    link.click();
    link.remove();
    URL.revokeObjectURL(url);
}

// Returns true if the current hash URL has query parameters active.
export function hasFilters() {
    return Boolean(document.location.hash.split("?")[1]);
}

// Clears any active query paramerters from the hash URL. 
export function clearHashQuery() {
    document.location.hash = document.location.hash.split("?")[0];
}

// Updates a hash URL query parameter.
export function setHashQueryParams(hashBaseUrl, name, value) {
    // Extract and parse hash query params
    const hashParams = document.location.hash.split("?")[1] || "";
    const params = new URLSearchParams(hashParams);

    if (value) {
        params.set(name, value);
    } else {
        params.delete(name);
    }

    // Removes known incompatible custom time bounds when selecting days
    if (name === "days") {
        params.delete("before");
        params.delete("after");
    }

    // Build and set new updated query params
    const query = params.toString();
    document.location.hash = query ? `${hashBaseUrl}?${query}` : hashBaseUrl;
}

// Updates the current hash query params with before and after showing a time range dialog.
// Returns false false if the dialog is cancelled.
// Any existing competing "days" query parameter will be deleted.
export async function openCustomTimeDialog(before, after, hashBaseUrl) {
    const selected = await timeRangeDialog(
        before ? new Date(Number(before) * 1000) : undefined,
        after ? new Date(Number(after) * 1000) : undefined,
    );

    // Early out and return false in case the dialog was cancelled
    if (!selected) return false;

    const hashParams = document.location.hash.split("?")[1] || "";
    const params = new URLSearchParams(hashParams);
    params.delete("days"); // Prevent incompatible settings

    for (const [name, date] of [["before", selected.before], ["after", selected.after]]) {
        if (date) {
            params.set(name, Math.floor(date.getTime() / 1000).toString());
        } else {
            params.delete(name);
        }
    }

    const query = params.toString();
    document.location.hash = query ? `${hashBaseUrl}?${query}` : hashBaseUrl;
}

// Creates a tooltip text for a custom time range.
export function timeFilterTooltip(after, before) {
    const excludedBefore = after ? new Date(Number(after) * 1000).toLocaleString() : undefined;
    const excludedAfter = before ? new Date(Number(before) * 1000).toLocaleString() : undefined;
    if (excludedBefore && excludedAfter) return `All items before ${excludedBefore} and after ${excludedAfter} are excluded.`;
    if (excludedBefore) return `All items before ${excludedBefore} are excluded.`;
    if (excludedAfter) return `All items after ${excludedAfter} are excluded.`;
    return "";
}
