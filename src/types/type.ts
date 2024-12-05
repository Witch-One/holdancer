export interface Formation {
  id: number;
  start_time: number;
  end_time: number;
  dancer: Dancer[];
}

export interface Dancer {
  id: number;
  name: string;
  position: Position;
}
export interface Position {
  x: number;
  y: number;
}
