/**
 * Player Tool Executor
 * Handles media playback operations
 */

import { logger } from "../../../../../core/monitoring/core/logger";
import { ExecutorContext, BaseExecutor } from "../core/types";

export class PlayerExecutor implements BaseExecutor {
  private context: ExecutorContext;

  constructor(context: ExecutorContext) {
    this.context = context;
  }

  execute(action: string, params: Record<string, any>): any {
    const mediaId = params.media_id || "player";
    const mediaElement = this.context.componentState.get(`${mediaId}_element`) as
      | HTMLMediaElement
      | undefined;

    switch (action) {
      case "play":
        if (mediaElement) {
          mediaElement.play();
          this.context.componentState.set(`${mediaId}_playing`, true);
        }
        logger.debug("Media playing", { component: "PlayerExecutor", mediaId });
        return true;

      case "pause":
        if (mediaElement) {
          mediaElement.pause();
          this.context.componentState.set(`${mediaId}_playing`, false);
        }
        logger.debug("Media paused", { component: "PlayerExecutor", mediaId });
        return true;

      case "stop":
        if (mediaElement) {
          mediaElement.pause();
          mediaElement.currentTime = 0;
          this.context.componentState.set(`${mediaId}_playing`, false);
        }
        logger.debug("Media stopped", { component: "PlayerExecutor", mediaId });
        return true;

      case "next":
        this.context.componentState.set(
          `${mediaId}_trackIndex`,
          this.context.componentState.get(`${mediaId}_trackIndex`, 0) + 1
        );
        logger.debug("Next track", { component: "PlayerExecutor" });
        return true;

      case "previous":
        this.context.componentState.set(
          `${mediaId}_trackIndex`,
          Math.max(0, this.context.componentState.get(`${mediaId}_trackIndex`, 0) - 1)
        );
        logger.debug("Previous track", { component: "PlayerExecutor" });
        return true;

      case "seek":
        if (mediaElement) {
          mediaElement.currentTime = params.time;
        }
        return params.time;

      case "setVolume":
        if (mediaElement) {
          mediaElement.volume = params.volume / 100;
        }
        this.context.componentState.set(`${mediaId}_volume`, params.volume);
        return params.volume;

      default:
        return null;
    }
  }
}
