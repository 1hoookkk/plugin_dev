#pragma once
#include <cmath>
#include <algorithm>

namespace emu
{
    struct EnvelopeFollower
    {
        void prepare(double sampleRate)
        {
            sr = sampleRate;
            state = 0.0f;
        }

        void setAttackMs(float ms)  { attackMs = ms; }
        void setReleaseMs(float ms) { releaseMs = ms; }
        void setDepth(float d)      { depth = d; }

        float process(float input) noexcept
        {
            const float rect = std::abs(input);
            const float attackSec  = attackMs  * 0.001f;
            const float releaseSec = releaseMs * 0.001f;
            const float tau  = (rect > state ? attackSec : releaseSec);
            const float alpha = 1.0f - std::exp(-1.0f / std::max(1e-6f, tau * static_cast<float>(sr)));
            state += alpha * (rect - state);
            return std::clamp(state * depth, 0.0f, 1.0f);
        }

        void reset() noexcept { state = 0.0f; }

        double sr { 48000.0 };
        float  state { 0.0f };
        float  attackMs { 0.489f };
        float  releaseMs { 80.0f };
        float  depth { 0.945f };
    };
}
