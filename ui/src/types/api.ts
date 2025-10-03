/**
 * API Types and Zod Schemas
 * Type-safe communication with backend using Zod for runtime validation
 */

import { z } from "zod";

// ============================================================================
// WebSocket Message Schemas - Client to Server
// ============================================================================

export const ChatMessageSchema = z.object({
  type: z.literal("chat"),
  message: z.string(),
  context: z.record(z.string(), z.unknown()).optional(),
});

export const GenerateUIMessageSchema = z.object({
  type: z.literal("generate_ui"),
  message: z.string(),
  context: z.record(z.string(), z.unknown()).optional(),
});

export const PingMessageSchema = z.object({
  type: z.literal("ping"),
});

export const ClientMessageSchema = z.discriminatedUnion("type", [
  ChatMessageSchema,
  GenerateUIMessageSchema,
  PingMessageSchema,
]);

// ============================================================================
// WebSocket Message Schemas - Server to Client
// ============================================================================

export const SystemMessageSchema = z.object({
  type: z.literal("system"),
  message: z.string(),
  connection_id: z.union([z.string(), z.number()]).optional(),
  model: z.string().optional(),
  timestamp: z.number().optional(),
});

export const TokenMessageSchema = z.object({
  type: z.literal("token"),
  content: z.string(),
});

export const ThoughtMessageSchema = z.object({
  type: z.literal("thought"),
  content: z.string(),
  timestamp: z.number(),
});

export const ReasoningMessageSchema = z.object({
  type: z.literal("reasoning"),
  content: z.string(),
  timestamp: z.number(),
});

export const GenerationTokenMessageSchema = z.object({
  type: z.literal("generation_token"),
  content: z.string(),
  timestamp: z.number().optional(),
});

export const ChatResponseMessageSchema = z.object({
  type: z.literal("chat_response"),
  content: z.string(),
  timestamp: z.number().optional(),
});

export const CompleteMessageSchema = z.object({
  type: z.literal("complete"),
  timestamp: z.number(),
});

export const GenerationStartMessageSchema = z.object({
  type: z.literal("generation_start"),
  message: z.string(),
  timestamp: z.number(),
});

export const UIComponentSchema: z.ZodType<any> = z.lazy(() =>
  z.object({
    type: z.string(),
    id: z.string(),
    props: z.record(z.string(), z.unknown()),
    children: z.array(UIComponentSchema).optional(),
    on_event: z.record(z.string(), z.string()).nullish(),
  })
);

export const UISpecSchema = z.object({
  type: z.string(),
  title: z.string(),
  layout: z.string(),
  components: z.array(UIComponentSchema),
  style: z.record(z.string(), z.unknown()).optional(),
  services: z.array(z.string()).optional(),
  service_bindings: z.record(z.string(), z.string()).optional(),
  lifecycle_hooks: z
    .object({
      on_mount: z.array(z.string()).optional(),
      on_unmount: z.array(z.string()).optional(),
    })
    .optional(),
});

export const UIGeneratedMessageSchema = z.object({
  type: z.literal("ui_generated"),
  app_id: z.string(),
  ui_spec: UISpecSchema,
  timestamp: z.number(),
});

export const ErrorMessageSchema = z.object({
  type: z.literal("error"),
  message: z.string(),
  timestamp: z.number().optional(),
});

export const ChatMessageItemSchema = z.object({
  role: z.enum(["user", "assistant", "system"]),
  content: z.string(),
  timestamp: z.number().optional(),
});

export const HistoryUpdateMessageSchema = z.object({
  type: z.literal("history_update"),
  history: z.array(ChatMessageItemSchema),
  timestamp: z.number(),
});

export const PongMessageSchema = z.object({
  type: z.literal("pong"),
});

export const ServerMessageSchema = z.discriminatedUnion("type", [
  SystemMessageSchema,
  TokenMessageSchema,
  ThoughtMessageSchema,
  ReasoningMessageSchema,
  GenerationTokenMessageSchema,
  ChatResponseMessageSchema,
  CompleteMessageSchema,
  GenerationStartMessageSchema,
  UIGeneratedMessageSchema,
  ErrorMessageSchema,
  HistoryUpdateMessageSchema,
  PongMessageSchema,
]);

// ============================================================================
// HTTP API Schemas
// ============================================================================

// Health Check
export const HealthResponseSchema = z.object({
  status: z.string(),
  model: z.string(),
  model_loaded: z.boolean(),
  active_connections: z.number(),
  gpu_enabled: z.boolean(),
  app_manager: z.record(z.string(), z.unknown()),
  kernel: z.object({
    connected: z.boolean(),
    default_pid: z.union([z.number(), z.null()]),
    system_info: z.record(z.string(), z.unknown()).optional(),
  }),
});

// Chat Request/Response
export const ChatRequestSchema = z.object({
  message: z.string(),
  context: z.record(z.string(), z.unknown()).optional(),
});

export const ChatResponseSchema = z.object({
  response: z.string(),
  thoughts: z.array(z.string()).optional(),
  ui_spec: z.record(z.string(), z.unknown()).nullable().optional(),
});

// Generate UI Response
export const GenerateUIResponseSchema = z.object({
  app_id: z.string().nullable(),
  ui_spec: UISpecSchema.nullable(),
  thoughts: z.array(z.string()).optional(),
  error: z.string().optional(),
});

// App Management
export const AppInfoSchema = z.object({
  id: z.string(),
  title: z.string(),
  state: z.string(),
  created_at: z.number(),
});

export const ListAppsResponseSchema = z.object({
  apps: z.array(AppInfoSchema),
  stats: z.record(z.string(), z.unknown()),
});

export const AppActionResponseSchema = z.object({
  success: z.boolean(),
  app_id: z.string(),
});

// Service Management
export const ServiceToolSchema = z.object({
  id: z.string(),
  name: z.string(),
  description: z.string(),
  params: z.record(z.string(), z.unknown()),
});

export const ServiceInfoSchema = z.object({
  id: z.string(),
  name: z.string(),
  description: z.string(),
  category: z.string(),
  version: z.string(),
  tools: z.array(ServiceToolSchema),
});

export const ListServicesResponseSchema = z.object({
  services: z.array(ServiceInfoSchema),
  stats: z.record(z.string(), z.unknown()),
});

export const DiscoverServicesRequestSchema = z.object({
  message: z.string(),
  context: z.record(z.string(), z.unknown()).optional(),
});

export const DiscoverServicesResponseSchema = z.object({
  query: z.string(),
  services: z.array(ServiceInfoSchema),
});

export const ServiceExecuteRequestSchema = z.object({
  tool_id: z.string(),
  params: z.record(z.string(), z.unknown()),
  app_id: z.string().optional(),
});

export const ServiceExecuteResponseSchema = z.object({
  success: z.boolean(),
  result: z.any().optional(),
  error: z.string().optional(),
});

// ============================================================================
// TypeScript Types (inferred from Zod schemas)
// ============================================================================

// Client Messages
export type ChatMessage = z.infer<typeof ChatMessageSchema>;
export type GenerateUIMessage = z.infer<typeof GenerateUIMessageSchema>;
export type PingMessage = z.infer<typeof PingMessageSchema>;
export type ClientMessage = z.infer<typeof ClientMessageSchema>;

// Server Messages
export type SystemMessage = z.infer<typeof SystemMessageSchema>;
export type TokenMessage = z.infer<typeof TokenMessageSchema>;
export type ThoughtMessage = z.infer<typeof ThoughtMessageSchema>;
export type CompleteMessage = z.infer<typeof CompleteMessageSchema>;
export type GenerationStartMessage = z.infer<typeof GenerationStartMessageSchema>;
export type UIComponent = z.infer<typeof UIComponentSchema>;
export type UISpec = z.infer<typeof UISpecSchema>;
export type UIGeneratedMessage = z.infer<typeof UIGeneratedMessageSchema>;
export type ErrorMessage = z.infer<typeof ErrorMessageSchema>;
export type ChatMessageItem = z.infer<typeof ChatMessageItemSchema>;
export type HistoryUpdateMessage = z.infer<typeof HistoryUpdateMessageSchema>;
export type PongMessage = z.infer<typeof PongMessageSchema>;
export type ServerMessage = z.infer<typeof ServerMessageSchema>;

// HTTP API Types
export type HealthResponse = z.infer<typeof HealthResponseSchema>;
export type ChatRequest = z.infer<typeof ChatRequestSchema>;
export type ChatResponse = z.infer<typeof ChatResponseSchema>;
export type GenerateUIResponse = z.infer<typeof GenerateUIResponseSchema>;
export type AppInfo = z.infer<typeof AppInfoSchema>;
export type ListAppsResponse = z.infer<typeof ListAppsResponseSchema>;
export type AppActionResponse = z.infer<typeof AppActionResponseSchema>;
export type ServiceTool = z.infer<typeof ServiceToolSchema>;
export type ServiceInfo = z.infer<typeof ServiceInfoSchema>;
export type ListServicesResponse = z.infer<typeof ListServicesResponseSchema>;
export type DiscoverServicesRequest = z.infer<typeof DiscoverServicesRequestSchema>;
export type DiscoverServicesResponse = z.infer<typeof DiscoverServicesResponseSchema>;
export type ServiceExecuteRequest = z.infer<typeof ServiceExecuteRequestSchema>;
export type ServiceExecuteResponse = z.infer<typeof ServiceExecuteResponseSchema>;

// ============================================================================
// Validation Helpers
// ============================================================================

/**
 * Safely parse and validate incoming server messages
 */
export function parseServerMessage(data: unknown): ServerMessage | null {
  try {
    return ServerMessageSchema.parse(data);
  } catch (error) {
    console.error("Failed to parse server message:", error);
    console.error("Received data:", data);
    return null;
  }
}

/**
 * Safely parse and validate outgoing client messages
 */
export function parseClientMessage(data: unknown): ClientMessage | null {
  try {
    return ClientMessageSchema.parse(data);
  } catch (error) {
    console.error("Failed to parse client message:", error);
    return null;
  }
}

/**
 * Create a type-safe client message
 */
export function createChatMessage(message: string, context?: Record<string, any>): ChatMessage {
  return ChatMessageSchema.parse({ type: "chat", message, context: context || {} });
}

export function createGenerateUIMessage(
  message: string,
  context?: Record<string, any>
): GenerateUIMessage {
  return GenerateUIMessageSchema.parse({ type: "generate_ui", message, context: context || {} });
}

export function createPingMessage(): PingMessage {
  return PingMessageSchema.parse({ type: "ping" });
}
