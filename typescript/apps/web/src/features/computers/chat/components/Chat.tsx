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
  ScrollArea,
} from "@mantine/core";
import { ComputerControls } from "../../components/ComputerControls";
import { profileStyle } from "../../profilePalette";
import classes from "./Chat.module.css";
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
  const working = props.writable && props.activity === "working";
  return (
    <AgentTheme palette={agent.paletteKey}>
      <div className={classes.frame}>
        <aside className={classes.sidebar}>
          {!props.navOpened ? <Navigation {...props} /> : null}
        </aside>
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
                  aria-label={`${agent.name} · ${props.status} · ${c.activity}`}
                >
                  <span className={classes.statusLine}>
                    <Icon name={working ? "activity" : "chat"} />
                    <strong>{agent.name}</strong>
                    <span>{props.status}</span>
                    <Icon name="chevron" />
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
              <span className={classes.srOnly} role="status">
                {props.status}
              </span>
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
              overscrollBehavior="contain"
              classNames={{
                viewport: classes.viewport,
                scrollbar: classes.scrollbar,
              }}
              viewportProps={{
                role: "log",
                "aria-label": c.message,
                tabIndex: 0,
              }}
            >
              <div className={classes.conversation} ref={f.contentRef}>
                {!props.showSample && props.messages.length === 0 ? (
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
                {props.showSample ? (
                  <>
                    <div className={classes.date}>{c.today}</div>
                    <div className={classes.message} data-sender="user">
                      <div className={classes.userBubble}>{c.user}</div>
                    </div>
                    <article className={classes.message} data-sender="agent">
                      <div className={classes.speaker}>
                        <CharacterArtwork
                          basePath={p.basePath}
                          assetKey={agent.profileImage.assetKey}
                          className={classes.speakerPortrait}
                        />
                        <strong>{agent.name}</strong>
                        <time>13:11</time>
                      </div>
                      <div className={classes.response}>
                        <p>{c.response}</p>
                        <section className={classes.plan}>
                          <div className={classes.planTitle}>
                            <div>
                              <span className={classes.eyebrow}>
                                {c.eyebrow}
                              </span>
                              <h2>{c.planTitle}</h2>
                              <p>{c.planSummary}</p>
                            </div>
                            <span className={classes.sun}>
                              <Icon name="sun" />
                            </span>
                          </div>
                          <div className={classes.route}>
                            {c.route.map((stop) => (
                              <div className={classes.routeRow} key={stop.time}>
                                <time>{stop.time}</time>
                                <div>
                                  <strong>{stop.title}</strong>
                                  <p>{stop.note}</p>
                                </div>
                              </div>
                            ))}
                          </div>
                          <div className={classes.planFooter}>
                            {c.planFooter}
                          </div>
                        </section>
                        <p className={classes.after}>{c.after}</p>
                      </div>
                    </article>
                    <div className={classes.message} data-sender="user">
                      <div className={classes.userBubble}>{c.followup}</div>
                    </div>
                  </>
                ) : null}
                {props.messages.map((message) => (
                  <div
                    className={classes.message}
                    data-sender="user"
                    key={message.id}
                  >
                    <div className={classes.userBubble}>{message.text}</div>
                  </div>
                ))}
              </div>
            </ScrollArea>
            <footer className={classes.composerArea} ref={f.overlayRef}>
              {f.detached ? (
                <ActionIcon
                  className={classes.latest}
                  size="2.25rem"
                  variant="default"
                  radius="xl"
                  aria-label={c.latest}
                  onClick={f.followLatest}
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
                      : `${props.status} · ${c.readOnly}`
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
                  <ActionIcon
                    variant="subtle"
                    size="2.25rem"
                    aria-label={c.attach}
                    disabled
                  >
                    <Icon name="plus" />
                  </ActionIcon>
                  <div className={classes.composerRight}>
                    <span>{c.codex}</span>
                    <ActionIcon
                      variant="filled"
                      className={classes.send}
                      size="2.25rem"
                      aria-label={c.send}
                      disabled={!props.writable || !props.input.trim()}
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
        <Drawer
          opened={props.opened}
          returnFocus={false}
          onExitTransitionEnd={props.focus.activityClosed}
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
          <div style={profileStyle(agent.paletteKey)}>
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
            <div className={classes.summary}>
              <span>
                {agent.name} · {c.codex}
              </span>
              <h2>
                {working
                  ? c.working
                  : props.activity === "stopped"
                    ? c.stopped
                    : c.activity}
              </h2>
              <p>{working ? c.taskDescription : props.status}</p>
            </div>
            <div className={classes.taskHeading}>
              <h3>{c.taskTitle}</h3>
              <span>{working ? "1 / 3" : "—"}</span>
            </div>
            <ol className={classes.tasks}>
              {c.tasks.map((task, index) => (
                <li
                  key={task}
                  data-current={(working && index === 1) || void 0}
                >
                  <Icon name={index === 0 ? "check" : "activity"} />
                  <span>{task}</span>
                </li>
              ))}
            </ol>
            <details className={classes.tools} open>
              <summary>
                {c.tools}
                <span>3</span>
              </summary>
              {[
                { title: c.search, text: c.query },
                { title: c.read, text: c.page },
                { title: c.read, text: c.hours },
              ].map((tool, index) => (
                <div className={classes.tool} key={tool.text}>
                  <div>
                    <Icon
                      name={index === 2 && working ? "activity" : "check"}
                    />
                    <strong>{tool.title}</strong>
                    <time>13:{index === 2 ? "13" : "12"}</time>
                  </div>
                  <p>{tool.text}</p>
                  <small>
                    {index === 2
                      ? working
                        ? c.inProgress
                        : c.stopped
                      : c.complete}
                  </small>
                </div>
              ))}
            </details>
            {working ? (
              <Button
                fullWidth
                variant="default"
                onClick={props.onStop}
                leftSection={<Icon name="stop" />}
              >
                {c.stop}
              </Button>
            ) : null}
          </div>
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
      </div>
    </AgentTheme>
  );
}
