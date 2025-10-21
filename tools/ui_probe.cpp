#include "../plugins/EngineField/Source/FieldProcessor.h"
#include <juce_audio_basics/juce_audio_basics.h>
#include <juce_audio_processors/juce_audio_processors.h>
#include <iostream>

int main()
{
    FieldProcessor processor;
    const double sampleRate = 48000.0;
    const int blockSize = 256;

    processor.prepareToPlay(sampleRate, blockSize);

    juce::AudioBuffer<float> buffer(2, blockSize);
    juce::MidiBuffer midi;

    // Fill with simple sine wave
    for (int i = 0; i < blockSize; ++i)
    {
        float sample = std::sin(juce::MathConstants<float>::twoPi * i / blockSize);
        buffer.setSample(0, i, sample);
        buffer.setSample(1, i, sample);
    }

    processor.processBlock(buffer, midi);

    std::vector<float> waveform(256);
    const int read = processor.getWaveformSamples(waveform.data(), static_cast<int>(waveform.size()));

    std::cout << "Waveform samples read: " << read << std::endl;
    if (read > 0)
    {
        std::cout << "First value: " << waveform.front() << ", last value: " << waveform[read - 1] << std::endl;
    }
    else
    {
        std::cout << "No waveform data available.\n";
    }
    return 0;
}
