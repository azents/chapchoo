"use client";
import { AgentTheme } from "@chapchoo/ui/AgentTheme";
import { CharacterArtwork } from "@chapchoo/ui/CharacterArtwork";
import { ConfirmationDialog } from "@chapchoo/ui/ConfirmationDialog";
import { ContextSidebar } from "@chapchoo/ui/ContextSidebar";
import { PageFailure } from "@chapchoo/ui/PageFailure";
import { agentPath } from "@chapchoo/web-core/workspace";
import {
  ActionIcon,
  Alert,
  Box,
  Button,
  Drawer,
  Group,
  Loader,
  ScrollArea,
} from "@mantine/core";
import { ComputerControls } from "../../components/ComputerControls";
import { profileStyle } from "../../profilePalette";
import { nativeStatus } from "../copy";
import classes from "./Chat.module.css";
import { ConversationBody, ConversationContent } from "./ConversationBody";
import { RecentConversationRecord } from "./ConversationRecord";
import type { ChatProps } from "../types";
import type { ReactElement } from "react";

const paths = {
  plus: "M12 5v14M5 12h14",
  arrow: "M12 19V5m-6 6 6-6 6 6",
  menu: "M4 6h16M4 12h16M4 18h16",
  chevron: "m9 5 7 7-7 7",
  chat: "M20 11a8 8 0 0 1-8 8H4l1-5a8 8 0 1 1 15-3Z",
  activity: "M3 12h4l3-7 4 14 3-7h4",
  check: "m5 12 4 4L19 6",
  more: "M5 12h.01M12 12h.01M19 12h.01",
  stop: "M6 6h12v12H6Z",
  sun: "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8M12 2v2m0 16v2M2 12h2m16 0h2",
};
function Icon({ name }: { name: keyof typeof paths }): ReactElement {
  return (
    <svg
      width="1.25rem"
      height="1.25rem"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.6"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d={paths[name]} />
    </svg>
  );
}

function Navigation(props: ChatProps): ReactElement {
  return (
    <ContextSidebar
      label={props.navigation?.label ?? props.product.copy.agents}
      header={props.navigation?.selector ?? props.navigation?.header}
      content={props.agentNavigation}
      items={props.navigation?.items ?? []}
      onSelect={props.onCloseNav}
    />
  );
}

export function Chat(props: ChatProps): ReactElement {
  const { product: p, copy: c, floating: f, agent } = props;
  if (p.access !== "authorized") {
    return (
      <PageFailure
        title={p.copy.agents}
        message={p.copy.accessUnavailable}
        appearance={p.appearance}
        retryLabel={p.access === "signedOut" ? p.copy.signIn : p.copy.retry}
        onRetry={p.onRefresh}
      />
    );
  }
  const detailHref = agentPath(p.basePath, p.data.teamHandle, agent.handle);
  const start = agent.computer.actions.find((item) => item.action === "start");
  const working = props.stoppable && props.activity === "working";
  const records = props.conversation.region.records;
  const currentPlan = props.conversation.region.latestPlan;
  const showStatus =
    props.statusLoading ||
    props.activity !== "idle" ||
    p.stale ||
    Boolean(p.error) ||
    agent.computer.power !== "running" ||
    agent.computer.connector !== "connected";
  return (
    <div className={classes.frame}>
      <aside className={classes.sidebar}>
        {!props.navOpened ? <Navigation {...props} /> : null}
      </aside>
      <AgentTheme palette={agent.paletteKey}>
        <main
          className={classes.workspace}
          style={profileStyle(agent.paletteKey)}
          aria-label={agent.name}
        >
          <header className={classes.presenceHeader}>
            <div className={classes.left}>
              <ActionIcon
                variant="subtle"
                size="2.75rem"
                className={classes.glass}
                aria-label={c.menu}
                ref={props.focus.navigationRef}
                onClick={props.onOpenNav}
              >
                <Icon name="menu" />
              </ActionIcon>
            </div>
            <div className={classes.presence} ref={f.profileRef}>
              <a
                className={`${classes.portrait} ${classes.glass}`}
                href={detailHref}
                aria-label={`${agent.name} · ${c.detail}`}
              >
                <CharacterArtwork
                  basePath={p.basePath}
                  assetKey={agent.profileImage.assetKey}
                  className={classes.artwork}
                />
              </a>
              <div className={`${classes.bubble} ${classes.glass}`}>
                <button
                  className={classes.status}
                  ref={props.focus.statusRef}
                  onClick={props.onOpen}
                  aria-haspopup="dialog"
                  aria-label={`${agent.name}${showStatus ? ` · ${props.status}` : ""} · ${c.activity}`}
                >
                  <span className={classes.statusLine}>
                    {showStatus && !props.statusLoading ? (
                      <Icon name={working ? "activity" : "chat"} />
                    ) : null}
                    <strong>{agent.name}</strong>
                    {showStatus ? <span>{props.status}</span> : null}
                    {props.statusLoading ? (
                      <Loader size="xs" aria-label={c.loading} />
                    ) : null}
                  </span>
                </button>
                {agent.computer.power === "stopped" ? (
                  <button
                    className={classes.power}
                    disabled={p.stale || p.pending || !start?.available}
                    onClick={() => p.onAction("start")}
                  >
                    {p.copy.start}
                  </button>
                ) : null}
              </div>
              {showStatus ? (
                <span className={classes.srOnly} role="status">
                  {props.status}
                </span>
              ) : null}
            </div>
            <div className={classes.right}>
              {props.accountControls}
              <ActionIcon
                component="a"
                href={detailHref}
                variant="subtle"
                size="2.75rem"
                aria-label={c.detail}
                className={classes.more}
              >
                <Icon name="more" />
              </ActionIcon>
            </div>
          </header>
          <div className={classes.timelineShell} ref={f.shellRef}>
            <ScrollArea
              className={classes.scrollRoot}
              viewportRef={f.viewportRef}
              type="hover"
              overscrollBehavior="auto"
              classNames={{
                viewport: classes.viewport,
                scrollbar: classes.scrollbar,
                content: classes.scrollContent,
              }}
              viewportProps={{
                role: "log",
                "aria-label": c.message,
                tabIndex: 0,
              }}
            >
              <div className={classes.conversation} ref={f.contentRef}>
                {props.conversation.read === "ready" &&
                props.conversation.region.header.state !== "loading" &&
                records.length === 0 &&
                props.conversation.inputs.length === 0 &&
                !props.conversation.region.inputs?.length ? (
                  <div className={classes.empty}>
                    <CharacterArtwork
                      basePath={p.basePath}
                      assetKey={agent.profileImage.assetKey}
                      className={classes.emptyPortrait}
                    />
                    <h1>{agent.name}</h1>
                    <p>{c.empty}</p>
                  </div>
                ) : null}
                <ConversationBody
                  controls={props.conversation}
                  copy={c}
                  writable={props.writable}
                  agentName={agent.name}
                  agentProfile={{
                    basePath: p.basePath,
                    assetKey: agent.profileImage.assetKey,
                  }}
                />
              </div>
            </ScrollArea>
            <footer className={classes.composerArea} ref={f.overlayRef}>
              {f.unseen ? (
                <ActionIcon
                  className={classes.latest}
                  size="2.25rem"
                  variant="default"
                  radius="xl"
                  aria-label={c.latest}
                  onClick={props.conversation.onLatest}
                >
                  <Box style={{ transform: "rotate(180deg)", display: "flex" }}>
                    <Icon name="arrow" />
                  </Box>
                </ActionIcon>
              ) : null}
              <div
                className={classes.composer}
                data-readonly={!props.writable || void 0}
              >
                <textarea
                  ref={f.textareaRef}
                  rows={1}
                  value={props.input}
                  disabled={!props.writable}
                  aria-label={c.message}
                  placeholder={
                    props.writable
                      ? `${agent.name} · ${c.input}`
                      : [props.status, c.readOnly].filter(Boolean).join(" · ")
                  }
                  onChange={(event) => props.onInput(event.target.value)}
                  onKeyDown={(event) => {
                    if (
                      event.key === "Enter" &&
                      !event.shiftKey &&
                      !event.nativeEvent.isComposing
                    ) {
                      event.preventDefault();
                      props.onSend();
                    }
                  }}
                />
                <div className={classes.composerActions}>
                  <div className={classes.composerRight}>
                    <span>{c.codex}</span>
                    <ActionIcon
                      variant="filled"
                      className={classes.send}
                      size="2.25rem"
                      aria-label={c.send}
                      disabled={!props.sendable}
                      onClick={props.onSend}
                    >
                      <Icon name="arrow" />
                    </ActionIcon>
                  </div>
                </div>
              </div>
            </footer>
          </div>
        </main>
      </AgentTheme>
      <Drawer
        opened={props.navOpened}
        returnFocus={false}
        onExitTransitionEnd={props.focus.navigationClosed}
        closeOnEscape={!props.secondaryOverlay}
        closeOnClickOutside={!props.secondaryOverlay}
        trapFocus={!props.secondaryOverlay}
        onClose={props.onCloseNav}
        position="left"
        size="18.125rem"
        title={props.navigation?.label ?? c.menu}
        closeButtonProps={{ "aria-label": c.close, size: "2.75rem" }}
      >
        <Navigation {...props} />
      </Drawer>
      <AgentTheme palette={agent.paletteKey}>
        <Drawer
          opened={props.opened}
          returnFocus={false}
          // Restore the status opener when this sheet returns to the transcript.
          onExitTransitionEnd={(): void => {
            if (props.historyOpened) {
              return;
            }
            props.focus.activityClosed();
          }}
          closeOnEscape={!p.confirmation && !p.detailsOpened}
          closeOnClickOutside={!p.confirmation && !p.detailsOpened}
          trapFocus={!p.confirmation && !p.detailsOpened}
          onClose={props.onClose}
          position="bottom"
          size="min(78dvh, 46rem)"
          title={c.activity}
          closeButtonProps={{ "aria-label": c.close }}
          classNames={{ content: classes.panel, body: classes.panelBody }}
          overlayProps={{ backgroundOpacity: 0.12, blur: 0 }}
        >
          <div
            className={classes.activityScroll}
            role="region"
            aria-label={c.activity}
            style={profileStyle(agent.paletteKey)}
          >
            <Group justify="flex-end" mb="md">
              <Button
                component="a"
                href={detailHref}
                variant="subtle"
                rightSection={<Icon name="chevron" />}
              >
                {c.detail}
              </Button>
            </Group>
            {p.stale ? (
              <Alert color="yellow" mb="lg">
                {p.copy.stale}
              </Alert>
            ) : null}
            {p.error && !p.confirmation ? (
              <Alert color="red" mb="lg">
                {p.error}
              </Alert>
            ) : null}
            <ComputerControls
              {...p}
              agent={agent}
              onDetailsExit={props.focus.nestedClosed}
            />
            {props.supplyControl}
            <div className={classes.summary}>
              <span>
                {agent.name} · {c.codex}
              </span>
              <h2>{c.execution}</h2>
              <p>{props.status}</p>
              {props.activity === "authenticationRequired" ? (
                <Button component="a" href={detailHref} variant="default">
                  {c.authenticationSettings}
                </Button>
              ) : props.activity === "startFailed" ||
                props.activity === "sourceUnavailable" ||
                props.activity === "retirementUnknown" ||
                props.activity === "unavailable" ? (
                <Button variant="default" onClick={props.conversation.onRead}>
                  {c.retry}
                </Button>
              ) : null}
              {props.conversation.stop !== "idle" ? (
                <p role="status">{c[props.conversation.stop]}</p>
              ) : null}
              {props.conversation.region.header.workId ? (
                <Button
                  variant="default"
                  onClick={props.onStop}
                  disabled={
                    !props.stoppable || props.conversation.stop !== "idle"
                  }
                  leftSection={<Icon name="stop" />}
                >
                  {c.stop}
                </Button>
              ) : null}
            </div>
            {currentPlan ? (
              <div className={classes.taskHeading}>
                <h3>{c.plan}</h3>
              </div>
            ) : null}
            {currentPlan?.plan?.type === "items" ? (
              <ol className={classes.tasks}>
                {currentPlan.plan.items.map((item, index) => (
                  <li key={index}>
                    <span>
                      {item.content} · {nativeStatus(c, item.status)}
                    </span>
                  </li>
                ))}
              </ol>
            ) : currentPlan?.plan?.type === "text" ? (
              <ConversationContent
                parts={[{ type: "text", text: currentPlan.plan.text }]}
              />
            ) : currentPlan ? (
              <ConversationContent parts={currentPlan.content} />
            ) : null}
            <section className={classes.tools}>
              <Button
                variant="subtle"
                fullWidth
                rightSection={<Icon name="chevron" />}
                onClick={() => props.conversation.onHistory()}
                classNames={{ inner: classes.historyLinkInner }}
              >
                {c.history}
              </Button>
              <div
                className={
                  records.length > 5 || props.conversation.region.older
                    ? classes.recentFade
                    : void 0
                }
              >
                {records.slice(-5).map((record) => (
                  <RecentConversationRecord
                    key={record.key}
                    record={record}
                    copy={c}
                    agentName={agent.name}
                    onOpen={() => props.conversation.onHistory(record)}
                  />
                ))}
              </div>
            </section>
          </div>
        </Drawer>
        <Drawer
          opened={props.historyOpened}
          onClose={props.onCloseHistory}
          title={c.history}
          closeButtonProps={{ "aria-label": c.close }}
          position="bottom"
          size="100dvh"
          returnFocus={false}
          classNames={{
            content: classes.panel,
            body: classes.historyPanelBody,
          }}
        >
          {props.historyContent}
        </Drawer>
        <ConfirmationDialog
          opened={p.confirmation !== null}
          title={
            p.confirmation === "force"
              ? p.copy.force
              : p.confirmation
                ? p.copy[p.confirmation]
                : p.copy.confirm
          }
          message={
            p.confirmation === "force"
              ? p.copy.forceWarning
              : p.confirmation === "delete"
                ? p.copy.deleteWarning
                : p.confirmation === "retryPreparation"
                  ? p.copy.retryPreparationWarning
                  : p.copy.stopWarning
          }
          confirmLabel={p.copy.confirm}
          cancelLabel={p.copy.cancel}
          onConfirm={p.onConfirm}
          onCancel={p.onCancel}
          onExitTransitionEnd={props.focus.nestedClosed}
          destructive={
            p.confirmation === "delete" || p.confirmation === "force"
          }
          state={
            p.pending
              ? { type: "PENDING" }
              : p.error
                ? { type: "ERROR", message: p.error }
                : { type: "READY" }
          }
        />
      </AgentTheme>
    </div>
  );
}
