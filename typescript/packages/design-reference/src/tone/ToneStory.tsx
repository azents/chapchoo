import { lazy, Suspense } from "react";
import type { ReactElement } from "react";
import type { ToneExample } from "./ToneDocument";
import type { ReferencePlatform } from "@chapchoo/ui/designTokens";

const ToneDocument = lazy(
  (): Promise<typeof import("./ToneDocument")> => import("./ToneDocument"),
);

/** Isolate the selected theme and Mantine reset from neutral ownership stories. */
export function ToneStory({
  example,
  platform = "web",
}: {
  example: ToneExample;
  platform?: ReferencePlatform;
}): ReactElement {
  const params = new URLSearchParams(window.location.search);
  if (params.has("tone-document")) {
    return (
      <Suspense fallback={<p>Loading reference…</p>}>
        <ToneDocument example={example} platform={platform} />
      </Suspense>
    );
  }
  params.set("tone-document", "");
  return (
    <iframe
      title={`Chapchoo ${example} / ${platform} reference`}
      src={`./iframe.html?${params.toString()}`}
      style={{ display: "block", width: "100%", height: "100dvh", border: 0 }}
    />
  );
}
