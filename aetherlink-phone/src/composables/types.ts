// types.ts
export interface Server {
  id: string;
  name: string;
  ip: string;
  port: number;
}

export interface Profile {
  id: string;
  name: string;
  description?: string;
}

export interface DevStatus {
  is_dev: boolean;
  mode: string;
}

export interface NewTask {
  name: string;
  description: string;
  path: string;
  type: string;
}
