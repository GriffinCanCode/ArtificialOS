/// <reference types="vite/client" />

// Node.js type declarations for compatibility
declare namespace NodeJS {
  type Timeout = ReturnType<typeof setTimeout>;
}

declare const process: {
  env: {
    NODE_ENV: string;
    [key: string]: string | undefined;
  };
};

