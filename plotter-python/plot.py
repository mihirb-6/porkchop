import matplotlib.dates as mdates
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
from matplotlib.lines import Line2D

plt.style.use("dark_background")

df_type1 = pd.read_csv("TYPEI_DATA.csv", header=0)
df_type2 = pd.read_csv("TYPEII_DATA.csv", header=0)

# x_type1,y_type1 => dtype = datetime64[ns] (numpy)
x_type1 = pd.to_datetime(df_type1["Departure Date [JD]"], origin="julian", unit="D")
y_type1 = pd.to_datetime(df_type1["Arrival Date [JD]"], origin="julian", unit="D")
x_type2 = pd.to_datetime(df_type2["Departure Date [JD]"], origin="julian", unit="D")
y_type2 = pd.to_datetime(df_type2["Arrival Date [JD]"], origin="julian", unit="D")

# Convert BEFORE passing to tricontour
x_type1_num = mdates.date2num(x_type1.dt.to_pydatetime())
y_type1_num = mdates.date2num(y_type1.dt.to_pydatetime())
x_type2_num = mdates.date2num(x_type2.dt.to_pydatetime())
y_type2_num = mdates.date2num(y_type2.dt.to_pydatetime())

# TOF
tof_type1 = y_type1_num - x_type1_num
tof_type2 = y_type2_num - x_type2_num


# Type I and II Departure and Arrival Energies (C3) [km^2/s^2]
C3_dep_type1 = df_type1["Departure C3 [km^2/s^2]"]
C3_arr_type1 = df_type1["Arrival C3 [km^2/s^2]"]
C3_dep_type2 = df_type2["Departure C3 [km^2/s^2]"]
C3_arr_type2 = df_type2["Arrival C3 [km^2/s^2]"]

# Type I and II Departure and Arrival Delta-V [km/s]
deltaV_dep_type1 = np.sqrt(C3_dep_type1)
deltaV_arr_type1 = np.sqrt(C3_arr_type1)
deltaV_dep_type2 = np.sqrt(C3_dep_type2)
deltaV_arr_type2 = np.sqrt(C3_arr_type2)

tofs = np.array([tof_type1, tof_type1, tof_type2, tof_type2])

delta_vs = np.array(
    [deltaV_dep_type1, deltaV_arr_type1, deltaV_dep_type2, deltaV_arr_type2]
)


def plot_tof_vs_DV(pmin: float, pmax: float):
    # Temporary array that will be filld in clipped values from all delta_vs
    temp = []

    # Clipping
    for i in range(len(delta_vs)):
        min = np.percentile(delta_vs[i], pmin)
        max = np.percentile(delta_vs[i], pmax)
        cap = np.clip(delta_vs[i], a_min=min, a_max=max)
        temp.append(cap)

    # Store in numpy arr
    capped_delta_vs = np.array(temp)

    # Create fig + ax objects
    fig, axs = plt.subplots(2, 2)

    # Flatten axes for easier plotting
    flat_axs = axs.flatten()

    # Scatter plot
    for i in range(len(capped_delta_vs)):
        ax = flat_axs[i]

        ax.scatter(tofs[i], capped_delta_vs[i], s=0.1, color=f"C{i}")
        ax.grid(True, linestyle="dotted", alpha=0.3)

    # Formatting
    fig.suptitle("TOF vs. Delta-V", weight="bold")
    fig.supylabel("Delta-V [km/s]", weight="bold")
    fig.supxlabel("Time of Flight (TOF) [Days]", weight="bold")
    # Custom legend format
    custom_lines = [
        Line2D([0], [0], color="C1", lw=1),
        Line2D([0], [0], color="C2", lw=1),
        Line2D([0], [0], color="C3", lw=1),
        Line2D([0], [0], color="C4", lw=1),
    ]
    fig.legend(
        custom_lines,
        ["Departure Type I", "Arrival Type I", "Departure Type II", "Arrival Type II"],
        ncol=4,
        loc="lower center",
        fontsize=5,
        bbox_to_anchor=(0.5, 0.05),
    )
    fig.tight_layout()
    fig.savefig("/Users/mihir/projects/porkchop/docs/tof_vs_deltaV.png", dpi=300)


def plot_contour(
    lw: float,
    dep_levels: np.ndarray,
    arr_levels: np.ndarray,
    dep_max_C3: float,
    arr_max_C3: float,
    plot_departure: bool = True,
    plot_arrival: bool = False,
    tof_contour: bool = False,
):

    # Departure C3 Type 1
    C3_dep_type1_cap = np.clip(C3_dep_type1, a_min=None, a_max=dep_max_C3)
    # Departure C3 Type 2
    C3_dep_type2_cap = np.clip(C3_dep_type2, a_min=None, a_max=dep_max_C3)

    # Arrival C3 Type 1
    C3_arr_type1_cap = np.clip(C3_arr_type1, a_min=None, a_max=arr_max_C3)
    # Arrival C3 Type 2
    C3_arr_type2_cap = np.clip(C3_arr_type2, a_min=None, a_max=arr_max_C3)

    # Create fig + ax objects
    fig, ax = plt.subplots()

    if plot_departure:
        # Type 1 Departure Energy Contour Lines
        type1_dep_lines = ax.tricontour(
            x_type1_num,  # <-- original floats from Julian Date
            y_type1_num,
            C3_dep_type1_cap,
            levels=dep_levels,
            cmap="gnuplot2_r",
            linewidths=lw,
        )
        ax.clabel(type1_dep_lines, inline=True, fontsize=4, fmt="%.0f", colors="white")

        # Type 2 Departure Energy Contour Lines
        type2_dep_lines = ax.tricontour(
            x_type2_num,
            y_type2_num,
            C3_dep_type2_cap,
            levels=dep_levels,
            cmap="gnuplot2_r",
            linewidths=lw,
        )
        ax.clabel(type2_dep_lines, inline=True, fontsize=4, fmt="%.0f", colors="white")

    if plot_arrival:
        # Type 1 Arrival Energy Contour Lines
        type1_arr_lines = ax.tricontour(
            x_type1_num,
            y_type1_num,
            C3_arr_type1_cap,
            levels=arr_levels,
            cmap="YlOrRd",
            linewidths=lw,
            alpha=0.99,
        )
        ax.clabel(type1_arr_lines, inline=True, fontsize=4, fmt="%.0f", colors="white")

        # Type 2 Arrival Energy Contour Lines
        type2_arr_lines = ax.tricontour(
            x_type1_num,
            y_type1_num,
            C3_arr_type2_cap,
            levels=arr_levels,
            cmap="YlOrRd",
            linewidths=lw,
            alpha=0.99,
        )
        ax.clabel(type2_arr_lines, inline=True, fontsize=4, fmt="%.0f", colors="white")

    if tof_contour:
        # TOF Contours Lines
        ax.set_xlim(x_type1_num.min(), x_type1_num.max())  # set limits
        ax.set_ylim(y_type1_num.min(), y_type1_num.max())
        for tof in np.arange(100, 600, 100):
            y_line = x_type1_num + tof  # y = x + TOF (constant TOF diagonal)
            ax.plot(x_type1_num, y_line, "white", lw=1, alpha=0.5)
            ax.annotate(
                str(int(tof)),
                xy=(x_type1_num[-1] + 10, y_line[-1] + 10),
                ha="center",
                va="center",
                annotation_clip=False,
                color="white",
                fontsize=10,
            )

    # Axis Date Formatting
    ax.xaxis.set_major_locator(mdates.MonthLocator(interval=1))
    ax.xaxis.set_major_formatter(mdates.DateFormatter("%d-%b-%Y"))

    ax.yaxis.set_major_locator(mdates.MonthLocator(interval=3))
    ax.yaxis.set_major_formatter(mdates.DateFormatter("%d-%b-%Y"))

    # Axis Label Formatting
    ax.tick_params(axis="x", rotation=45, labelsize=10)
    ax.tick_params(axis="y", labelsize=10)

    # Plot Formatting
    plt.title("Ballistic Transfer Trajectories", weight="bold")
    plt.xlabel("Departure Date", weight="bold")
    plt.ylabel("Arrival Date", weight="bold")
    fig.autofmt_xdate(rotation=45, ha="right")
    plt.grid(True, linestyle="dotted", alpha=0.3)

    # Custom legend format
    custom_lines = [
        Line2D([0], [0], lw=1),
        Line2D([0], [0], lw=1),
    ]

    if plot_arrival:
        plt.legend(
            custom_lines,
            ["Departure C3 [${km^2}/{s^2}$]", "Arrival C3 [${km^2}/{s^2}$]"],
        )
    else:
        plt.legend(custom_lines[:1], [r"Departure C3 [${km^2}/{s^2}$]"])
    plt.tight_layout()

    plt.savefig("/Users/mihir/projects/porkchop/docs/porkchop_plot.png", dpi=300)


if __name__ == "__main__":
    departure_levels = np.linspace(0, 100, 20)
    arrival_levels = np.linspace(0, 100, 15)
    linewidth = 0.5
    max_C3 = 64

    plot_contour(
        linewidth,
        departure_levels,
        arrival_levels,
        max_C3,
        max_C3,
        plot_departure=True,
        plot_arrival=False,
    )

    # plot_tof_vs_DV(pmin=10, pmax=40)
