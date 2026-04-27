import worldCubeUrl from "@/assets/home/world-cube.png";
import clockUrl from "@/assets/home/clock.svg";
import figureUrl from "@/assets/home/figure.png";
import stickyNoteUrl from "@/assets/home/sticky-note.svg";

export function HomeView() {
  return (
    <div className="relative h-full w-full overflow-hidden">
      <img
        src={worldCubeUrl}
        alt=""
        aria-hidden="true"
        className="pointer-events-none absolute left-[35%] top-[20%] w-[40%] max-w-[700px] select-none"
      />

      <img
        src={clockUrl}
        alt=""
        aria-hidden="true"
        className="pointer-events-none absolute right-[5%] top-[5%] w-[12%] max-w-[200px] select-none"
      />

      <img
        src={figureUrl}
        alt=""
        aria-hidden="true"
        className="pointer-events-none absolute left-[18%] top-[20%] w-[8%] max-w-[130px] select-none"
      />
      <img
        src={figureUrl}
        alt=""
        aria-hidden="true"
        className="pointer-events-none absolute right-[8%] top-[35%] w-[6%] max-w-[100px] select-none"
      />
      <img
        src={figureUrl}
        alt=""
        aria-hidden="true"
        className="pointer-events-none absolute left-[45%] top-[65%] w-[6%] max-w-[100px] select-none"
      />

      <img
        src={stickyNoteUrl}
        alt=""
        aria-hidden="true"
        className="pointer-events-none absolute bottom-[15%] left-[15%] w-[18%] max-w-[300px] select-none"
      />
    </div>
  );
}
