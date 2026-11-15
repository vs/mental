# What makes a moving picture clear

Notes for [the report](../report.md). The research on animation is less flattering than intuition, and it changes the design.

## Animation does not, by itself, help understanding

**Tversky, Morrison and Betrancourt (2002), "Animation: can it facilitate?"** Reviewing the studies, they found that animation is not a general improvement over good static graphics for explaining complex systems; many studies that seemed to show a benefit had given the animated group more information or interactivity. Two principles explain it. *Congruence*: the form of the picture should match the form of the idea (change over time shown as change over time). *Apprehension*: the picture must be perceived accurately, and animations often fail here because they are too fast and too complex to take in. Interactivity may overcome this ([paper](https://hci.stanford.edu/courses/cs448b/papers/Tversky_AnimationFacilitate_IJHCS02.pdf)).

**Robertson et al. (2008), trend animation.** For analysis, animation was the least effective form: both static forms (all traces overlaid, and small multiples) were faster, and small multiples were more accurate. Participants nevertheless found animation enjoyable ([Fisher's retrospective](https://medium.com/@FisherDanyel/effectiveness-of-animation-in-trend-visualization-ten-years-later-e2f52b433526)). A 2019 replication on phones found small multiples faster again, with accuracy depending on the task ([Brehmer et al.](https://arxiv.org/abs/1907.03919v1)).

**Hundhausen, Douglas and Stasko (2002), algorithm visualization.** A meta-study of 24 experiments: *how* students use a visualization matters more than *what* it shows. Predicting, answering questions, changing inputs and building one's own view beat watching ([DOI](https://doi.org/10.1006/jvlc.2002.0237)).

Consequences:

- Playback is for orientation and for pleasure. Answers come from stepping, asking "why", flipping a fork and comparing two runs. The viewer's interactive controls matter more than the beauty of the motion.
- Every animated view needs a **static companion** that holds the same run still: a trail of where the request has been, a storyboard of key moments, a time-space diagram.

## When animation does help

**Animated transitions (Heer and Robertson, 2007).** Animating the change between two views of the same data improves perception of what changed, and staged transitions (one kind of change at a time) are preferred, though staging did not clearly beat a single stage in accuracy ([paper](https://idl.uw.edu/papers/animated-transitions)). Tversky's review also allows that animation suits "real time reorientations in time and space".

So animation is right for: changing zoom on any axis, switching between views of the same trace, and moving the camera. In these the viewer must keep track of *which thing is which*, and motion does that.

## Limits of attention

People can track roughly four moving objects at once. The exact number depends on the display: with slow speeds and wide spacing it can reach eight, and speed and crowding reduce it ([overview](https://en.wikipedia.org/wiki/Multiple_object_tracking), [Alvarez and Franconeri 2007](https://visualcognition.psych.northwestern.edu/publications/AlvarezTracking2007.pdf)).

Consequences:

- "Follow the request" is clear because it has one subject. It obeys the limit by construction.
- A town with fifty moving things cannot be *followed*. It can only be *sensed*: where is it busy, where is it stuck. That is a legitimate use, and the design should be honest that it is the use. To follow something in the town, the viewer picks it and the script changes to a follow.
- A director should never move more than about four meaningful things at once; when the trace has more concurrency than that, it should slow down, stage the moves, or split the screen.

## Showing direction and flow on a line

Tapered and animated edges were compared for reading direction in node-link diagrams ([Holten, Isenberg and van Wijk 2011](https://www.lri.fr/~isenberg/publications/papers/Holten_2011_AEP.pdf)), and particles moving along edges have a described design space: pattern, frequency and speed can each carry a value ([Romat et al. 2018](https://hal.archives-ouvertes.fr/hal-01726358)). The detailed findings were not retrieved in this study.

Consequence: a single message is one token with its own identity. Aggregate flow (many requests) is a particle texture, and its speed or density then *means* something, so it must be mapped to a model fact or not be used.

## The danger of a metaphor: it says things the model does not

A town adds distance, size, travel time and weather. The model has none of them. If a message takes three seconds to walk across the square, the viewer learns "this call is slow", which may be false. This breaks the specification's presentation rules in spirit (an example value shown unmarked).

Rule proposed in the report: every visual channel of a stage or skin (position, distance, size, speed, color, sound) is either **bound** to a model fact or declared **decorative**, and the engine can list which is which. SimCity's "what you see is what we sim" is the same rule from the game side.

## Accessibility

Some people cannot use motion at all; browsers expose a "reduce motion" preference. The static companion (storyboard, stepping with cuts and no movement) is therefore not optional. Color must never be the only carrier of error, assumption or hypothetical status.
