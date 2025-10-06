/**
 * Game Tool Executor
 * Handles game state management
 */

import { logger } from "../../../../../core/utils/monitoring/logger";
import { ExecutorContext, BaseExecutor } from "../core/types";

export class GameExecutor implements BaseExecutor {
  private context: ExecutorContext;

  constructor(context: ExecutorContext) {
    this.context = context;
  }

  execute(action: string, params: Record<string, any>): any {
    switch (action) {
      case "move":
        const position = params.position;
        const gameState = this.context.componentState.get("game_state", {});

        // Update game state with move
        this.context.componentState.set("game_state", { ...gameState, lastMove: position });
        this.context.componentState.set(`cell_${position}`, params.data);

        logger.debug("Game move", { component: "GameExecutor", position });
        return true;

      case "reset":
        this.context.componentState.set("game_state", {});
        this.context.componentState.batch(() => {
          // Clear all cell states
          for (let i = 0; i < 9; i++) {
            this.context.componentState.set(`cell_${i}`, "");
          }
        });
        logger.debug("Game reset", { component: "GameExecutor" });
        return true;

      case "score":
        const currentScore = this.context.componentState.get(`score_${params.player}`, 0);
        this.context.componentState.set(`score_${params.player}`, currentScore + params.score);
        return currentScore + params.score;

      default:
        return null;
    }
  }
}
