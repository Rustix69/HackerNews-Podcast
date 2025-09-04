from google import genai
from google.genai import types
import wave
from google.colab import userdata

# Function to save PCM data into a .wav file
def wave_file(filename, pcm, channels=1, rate=24000, sample_width=2):
    with wave.open(filename, "wb") as wf:
        wf.setnchannels(channels)
        wf.setsampwidth(sample_width)
        wf.setframerate(rate)
        wf.writeframes(pcm)

# Fetch API key from Colab secrets or set manually
API_KEY = "AIzaSyA7VZt6MXOt10vEwbwjwpSrL49hlnsCVnc"  # <-- Replace with your actual API key

# Initialize the GenAI client
client = genai.Client(api_key=API_KEY)

# Long text content using triple quotes
contents = """
Étoilé was so promising…. the idea of using OpenStep (via GNUstep) infrastructure in ways that went beyond NeXTstep or Mac OS X. 
It felt like an even deeper embrace of the Smalltalk influences of the OpenStep API to enhance the desktop computing experience. 
Étoilé wasn’t merely a clone of NeXTstep or Mac OS X; it was its own thing that could’ve been a compelling contender to KDE, GNOME, and even macOS.

Unfortunately, Étoilé seems to have been inactive for a decade, and even Apple seems to be abandoning its Xerox PARC-inspired influences 
as the old guard retires and passes away (RIP Steve Jobs, Larry Tesler, and Bill Atkinson). Ever since Apple struck gold with the iPhone 
and derivative platforms, it’s as though the company deliberately shifted from user-empowering computing paradigms to user-enclosing ecosystems.

Back in the NeXTstep and early Mac OS X days, Apple drew heavily from object-oriented interface philosophies. The OpenStep API was designed 
around the idea that software objects mirrored real-world metaphors, and the developer experience was deeply tied to user experience. 
Applications weren’t just collections of windows; they were living systems, composed of reusable, dynamic objects that users could manipulate intuitively.

Étoilé tried to carry that torch forward. Its mission wasn’t just about building a new desktop environment — it wanted to redefine how we think about personal computing. 
While KDE and GNOME focused on Unix-like sensibilities, and macOS became increasingly closed, Étoilé was exploring concepts like:

- Semantic computing — applications that understand the meaning of the data you’re working with, not just the format.
- Unified workspaces — blurring the line between apps, documents, and processes.
- Dynamic object interaction — making the whole desktop feel like a Smalltalk playground where everything is an object you can introspect, manipulate, and extend.

It was ambitious, idealistic, and a little ahead of its time.

The Dream That Slipped Away

If you think about it, Étoilé represented one of the last serious attempts to push desktop metaphors beyond the WIMP paradigm (Windows, Icons, Menus, Pointer). 
NeXTstep itself was a reimagining of Xerox PARC’s ideas, but even it remained somewhat bound to the "desktop + folders + apps" worldview. 
Étoilé’s designers wanted to escape that box entirely.

They envisioned a world where your “documents” weren’t static files buried inside directories, but semantic objects floating within a knowledge space. 
Imagine clicking on a person’s name in an email and instantly seeing their entire history of interactions across projects, chats, documents, and more — 
without opening six different apps or digging through hierarchical folders.

That kind of integrated, semantic desktop is still rare today. Projects like GNOME’s Tracker, KDE’s Nepomuk, and Microsoft’s much-hyped but canceled WinFS 
all tried to move in that direction, but most fell short due to complexity, performance constraints, and lack of mainstream buy-in. 
Étoilé, with its OpenStep roots and Smalltalk-inspired philosophy, arguably had a cleaner conceptual foundation than all of them — but it never achieved critical mass.

Apple’s Divergence

Part of the tragedy is that Apple, once the torchbearer of user-centric computing, moved in the opposite direction. 
Instead of doubling down on object-oriented openness, it embraced walled gardens. 
The post-iPhone Apple became obsessed with simplicity at the cost of agency.

Whereas NeXTstep and early macOS gave you the feeling of being in control — like a craftsman with a rich set of tools — today’s Apple ecosystem often feels like a curated theme park. 
It’s beautiful, sure, but you’re not allowed backstage.

- You don’t get to peek under the hood anymore.
- You don’t get true scriptable system-wide interoperability.
- You don’t get deep customization without jailbreaking or hacks.

The irony is that, in the early 2000s, Mac OS X was the platform where developers, designers, and tinkerers felt empowered. 
Interface Builder, Objective-C, Cocoa — they gave you insane leverage. You could build something world-class with a tiny team 
because the frameworks were so composable and intuitive.

That ethos — “power to the individual hacker” — was exactly what Étoilé wanted to amplify. 
But in today’s Apple, that ethos has faded into the background.

The Rise of Thin Clients

While Étoilé dreamt of richer desktops, the rest of the world raced toward thin clients. Web apps replaced native apps. 
Chromebooks taught people to live entirely inside a browser. 
Even macOS and Windows increasingly nudge users toward cloud-first, subscription-driven ecosystems.

On the surface, this feels like progress — instant sync, universal access, platform independence — but something intangible has been lost. 
Desktop computing used to feel alive. You had rich, introspectable objects, modular frameworks, and the freedom to build workflows 
that matched your mental model, not the other way around.

Now? The browser tab is the new “app,” and you’re mostly just streaming interfaces from someone else’s server. 
It’s efficient, but sterile.

Étoilé, had it survived and thrived, might have offered an alternative path — one where local computation, semantic awareness, 
and personal agency stayed at the center of the experience.

Smalltalk’s Ghost

Underlying all of this is the shadow of Smalltalk, the language and environment that inspired so much of NeXTstep, OpenStep, and thus Étoilé. 
Smalltalk treated everything as an object in a living system. 
The environment itself was editable at runtime. 
You could inspect any object, modify it, and see the changes ripple instantly.

Imagine a desktop OS where that ethos was mainstream. 
Where your “apps” weren’t monolithic silos but collections of collaborating objects. 
Where you could pause execution, poke at the state of the system, extend functionality on the fly, and share modifications with others seamlessly.

That’s the computing future Étoilé quietly gestured toward — and it’s a future we’re still missing.
"""

# Generate audio response using Gemini 2.5 TTS
response = client.models.generate_content(
    model="gemini-2.5-flash-preview-tts",
    contents=contents,
    config=types.GenerateContentConfig(
        response_modalities=["AUDIO"],
        speech_config=types.SpeechConfig(
            voice_config=types.VoiceConfig(
                prebuilt_voice_config=types.PrebuiltVoiceConfig(
                    voice_name="Kore"
                )
            )
        ),
    ),
)

# Extract PCM audio data
data = response.candidates[0].content.parts[0].inline_data.data

# Save as .wav file
file_name = "etoile_tts.wav"
wave_file(file_name, data)

print(f"Audio saved as {file_name}")
