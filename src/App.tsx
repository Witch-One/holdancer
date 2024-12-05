import { useEffect, useRef, useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { Button, Chip } from "@nextui-org/react";
import dayjs from "dayjs";
import utc from "dayjs/plugin/utc";
import data from "./data.json";
import _, { findLast, set } from "lodash";
import { Stage, Layer, Circle } from "react-konva";
import { Formation } from "./types/type";
import { time } from "framer-motion/client";
dayjs.extend(utc);

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  const [shot, setShot] = useState<Formation>();
  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  const [currentTime, setCurrentTime] = useState(0);
  const startTime = useRef(0);
  const [isPlaying, setIsPlaying] = useState(false);
  const animationFrameRef = useRef<number | null>(null);

  const pause = () => {
    console.log("here pause");

    if (animationFrameRef.current) {
      console.log("here cancelAnimationFrame", animationFrameRef.current);

      cancelAnimationFrame(animationFrameRef.current);
      setIsPlaying(false);
    }
  };

  const playing = (timeStamp: number) => {
    if (timeStamp - startTime.current > timeLineLength) {
      console.log("here");

      setCurrentTime(timeLineLength);
      pause();
      return;
    }
    setCurrentTime(timeStamp - startTime.current);
    animationFrameRef.current = requestAnimationFrame(playing);
  };

  const handleClick = () => {
    setIsPlaying(true);
    startTime.current = Number(document.timeline.currentTime);
    animationFrameRef.current = requestAnimationFrame(playing);
  };

  const getCurrentTimeShot = (time: number) => {
    const shot = data.find((shot) => {
      return shot.start_time <= time && shot.end_time > time;
    });
    if (shot) {
      return shot;
    } else {
      const prevShot = _.findLast(data, (shot) => shot.end_time < time);
      const nextShot = _.find(data, (shot) => shot.start_time > time);
      if (prevShot && nextShot) {
        const progress =
          (time - prevShot.end_time) /
          (nextShot.start_time - prevShot.end_time);
        return {
          ...prevShot,
          dancer: prevShot.dancer.map((dancer) => {
            const prevDancer = prevShot.dancer.find((d) => d.id === dancer.id);
            const nextDancer = nextShot.dancer.find((d) => d.id === dancer.id);
            if (!prevDancer || !nextDancer) return dancer;
            return {
              ...dancer,
              position: {
                x:
                  prevDancer.position.x +
                  (nextDancer.position.x - prevDancer.position.x) * progress,
                y:
                  prevDancer.position.y +
                  (nextDancer.position.y - prevDancer.position.y) * progress,
              },
            };
          }),
        };
      }
    }
  };
  const timeLineLength = findLast(data)?.end_time || 0;

  const handleTimeLineScroll = () => {
    const scrollLeft = timeLineRef.current?.scrollLeft || 0;
    setCurrentTime(scrollLeft * 10);
  };

  // useEffect(() => {
  //   if (timeLineRef.current) {
  //     const shot = getCurrentTimeShot(currentTime);

  //     shot && setShot(shot);
  //   }
  // }, [currentTime]);

  useEffect(() => {
    try {
      invoke("get_formation", { ts: currentTime }).then((res) => {
        console.log(JSON.parse(res as string));
        setShot(JSON.parse(res as string));
        if (timeLineRef.current) {
          timeLineRef.current.scrollLeft = currentTime / 10;
        }
      });
    } catch (e) {
      console.error(e);
    }
  }, [currentTime]);

  const timeLineRef = useRef<HTMLDivElement>(null);

  const handleUpdateShot = async (shot: Formation) => {
    await invoke("update_formation", {
      ts: currentTime,
      formation: JSON.stringify(shot),
    });
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    console.log("here", e);
    if (e.code === "Space") {
      console.log("here", isPlaying);

      isPlaying ? pause() : handleClick();
    }
  };

  const handleUpdateDancerPosition = (
    id: number,
    position: { x: number; y: number }
  ) => {
    const newShot = {
      ...shot,
      dancer: shot?.dancer.map((dancer) => {
        if (dancer.id === id) {
          return {
            ...dancer,
            position,
          };
        }
        return dancer;
      }),
    };
    handleUpdateShot(newShot as Formation);
  };

  useEffect(() => {
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [handleKeyDown]);

  return (
    <main className="container">
      <h1>Welcome to Tauri + React</h1>
      <Stage width={400} height={400}>
        <Layer>
          {shot?.dancer.map((dancer) => (
            <Circle
              key={dancer.id}
              fill={"blue"}
              radius={10}
              draggable
              onDragMove={(e) => {
                console.log(e);
                handleUpdateDancerPosition(dancer.id, {
                  x: e.target.x(),
                  y: e.target.y(),
                });
              }}
              x={dancer.position.x}
              y={dancer.position.y}
            />
          ))}
        </Layer>
      </Stage>
      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />

        <button type="submit">Greet</button>
      </form>
      <Chip>{dayjs.utc(currentTime).format("HH:mm:ss.SSS")}</Chip>
      <Button
        className="relative overflow-hidden"
        onClick={isPlaying ? pause : handleClick}
      >
        {isPlaying ? "Pause" : "Play"}
      </Button>
      <div className="h-12 relative overflow-hidden  ">
        <div
          ref={timeLineRef}
          className="flex flex-row  h-full  overflow-x-auto no-scrollbar "
          onScroll={handleTimeLineScroll}
        >
          <span className="w-[50vw] h-full flex-shrink-0 " />
          <div
            className={`overflow-auto flex-shrink-0 h-full bg-gray-500`}
            style={{
              width: timeLineLength / 10 + "px",
            }}
          ></div>
          <span className="w-[50vw] h-full flex-shrink-0" />
        </div>
        <span className="absolute h-full w-[1px] bg-white left-[50%] top-0"></span>
      </div>
    </main>
  );
}

export default App;
